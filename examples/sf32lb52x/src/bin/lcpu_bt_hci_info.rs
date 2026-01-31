//! 测试 LCPU BT controller 的 HCI 命令支持情况。
//!
//! 依次发送多个 HCI 命令，验证 controller 是否正确响应。
//! 用于确认 controller 能力，为使用 trouble 库做准备。

#![no_std]
#![no_main]

use defmt::*;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_time::{Duration, Timer, WithTimeout};
use panic_probe as _;
use static_cell::StaticCell;

use bt_hci::cmd::controller_baseband::Reset;
use bt_hci::cmd::info::{ReadBdAddr, ReadLocalVersionInformation};
use bt_hci::cmd::le::{LeReadBufferSize, LeReadLocalSupportedFeatures};
use bt_hci::controller::{Controller, ControllerCmdSync, ExternalController};

use sifli_hal::bt_hci::IpcHciTransport;
use sifli_hal::lcpu::{Lcpu, LcpuConfig};
use sifli_hal::{bind_interrupts, ipc, rcc, syscfg};

bind_interrupts!(struct Irqs {
    MAILBOX2_CH1 => ipc::InterruptHandler;
});

type BtController = ExternalController<IpcHciTransport, 4>;
static CONTROLLER: StaticCell<BtController> = StaticCell::new();

/// HCI 事件读取任务
#[embassy_executor::task]
async fn hci_reader_task(controller: &'static BtController) {
    let mut buf = [0u8; 259];
    loop {
        match controller.read(&mut buf).await {
            Ok(_packet) => {
                trace!("HCI event received");
            }
            Err(_e) => {
                error!("HCI read error");
                Timer::after(Duration::from_millis(100)).await;
            }
        }
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = sifli_hal::init(Default::default());
    let rev = syscfg::read_idr().revision();

    // 启动 LCPU
    info!("Powering on LCPU...");
    let lcpu = Lcpu::new(p.LPSYS_AON);
    if let Err(e) = lcpu.power_on(&LcpuConfig::default()) {
        error!("LCPU power_on failed: {:?}", e);
        loop {
            Timer::after_secs(1).await;
        }
    }
    // 保持 LCPU 唤醒
    unsafe { rcc::wake_lcpu() };
    info!("LCPU is running");

    // 创建 IPC queue
    let mut ipc_driver = ipc::Ipc::new(p.MAILBOX1_CH1, Irqs, ipc::Config::default());
    let queue = match ipc_driver.open_queue(ipc::QueueConfig::qid0_hci(rev)) {
        Ok(q) => q,
        Err(e) => {
            error!("open_queue failed: {:?}", e);
            loop {
                Timer::after_secs(1).await;
            }
        }
    };
    info!("IPC queue opened");

    // 创建 Controller
    let transport = IpcHciTransport::new(queue);
    let controller = CONTROLLER.init(ExternalController::new(transport));

    // 启动 HCI reader task
    spawner.spawn(hci_reader_task(controller).expect("spawn hci_reader_task"));
    Timer::after(Duration::from_micros(1000)).await;

    info!("========== Testing HCI Commands ==========");

    // 1. HCI Reset
    info!("1. HCI Reset...");
    match controller
        .exec(&Reset::new())
        .with_timeout(Duration::from_secs(2))
        .await
    {
        Ok(Ok(_)) => info!("   OK"),
        Ok(Err(_e)) => error!("   FAILED"),
        Err(_) => error!("   TIMEOUT"),
    }

    // 2. Read Local Version Information
    info!("2. Read Local Version Information...");
    match controller
        .exec(&ReadLocalVersionInformation::new())
        .with_timeout(Duration::from_secs(2))
        .await
    {
        Ok(Ok(ret)) => {
            // 复制 packed struct 字段到局部变量
            let hci_ver = ret.hci_version;
            let hci_subver = ret.hci_subversion;
            let lmp_ver = ret.lmp_version;
            let company_id = ret.company_identifier;
            let lmp_subver = ret.lmp_subversion;
            info!("   HCI Version: {:?}", hci_ver);
            info!("   HCI Subversion: 0x{:04X}", hci_subver);
            info!("   LMP Version: {:?}", lmp_ver);
            info!("   Company ID: 0x{:04X}", company_id);
            info!("   LMP Subversion: 0x{:04X}", lmp_subver);
        }
        Ok(Err(_e)) => error!("   FAILED"),
        Err(_) => error!("   TIMEOUT"),
    }

    // 3. Read BD_ADDR
    info!("3. Read BD_ADDR...");
    match controller
        .exec(&ReadBdAddr::new())
        .with_timeout(Duration::from_secs(2))
        .await
    {
        Ok(Ok(addr)) => {
            info!(
                "   BD_ADDR: {:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}",
                addr.0[5], addr.0[4], addr.0[3], addr.0[2], addr.0[1], addr.0[0]
            );
        }
        Ok(Err(_e)) => error!("   FAILED"),
        Err(_) => error!("   TIMEOUT"),
    }

    // 4. LE Read Buffer Size
    info!("4. LE Read Buffer Size...");
    match controller
        .exec(&LeReadBufferSize::new())
        .with_timeout(Duration::from_secs(2))
        .await
    {
        Ok(Ok(ret)) => {
            let pkt_len = ret.le_acl_data_packet_length;
            let num_pkts = ret.total_num_le_acl_data_packets;
            info!("   LE ACL Data Packet Length: {}", pkt_len);
            info!("   Total Num LE ACL Data Packets: {}", num_pkts);
        }
        Ok(Err(_e)) => error!("   FAILED"),
        Err(_) => error!("   TIMEOUT"),
    }

    // 5. LE Read Local Supported Features
    info!("5. LE Read Local Supported Features...");
    match controller
        .exec(&LeReadLocalSupportedFeatures::new())
        .with_timeout(Duration::from_secs(2))
        .await
    {
        Ok(Ok(features)) => {
            // LeFeatureMask 实现了 Debug
            info!("   LE Features: {:?}", defmt::Debug2Format(&features));
        }
        Ok(Err(_e)) => error!("   FAILED"),
        Err(_) => error!("   TIMEOUT"),
    }

    info!("========== Test Complete ==========");

    loop {
        Timer::after_secs(1).await;
    }
}
