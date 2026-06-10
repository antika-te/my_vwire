#![no_std]
#![no_main]

use aya_ebpf::bindings::{XDP_REDIRECT, XDP_PASS, xdp_action};
use aya_ebpf::macros::{xdp, map};
use aya_ebpf::programs::XdpContext;
use aya_ebpf::maps::{XskMap, HashMap};
use aya_ebpf::helpers::{bpf_xdp_redirect_map, bpf_get_smp_processor_id};

// 定義常數
const ETH_P_IP: u16 = 0x0800;
const IPPROTO_TCP: u8 = 6;
const HTTP_PORT: u16 = 80;

// AF_XDP Socket Map - 用於重定向到 userspace
#[map]
static mut XSCK_MAP: XskMap = XskMap::with_max_entries(1, 0);

// 統計信息 Map
#[map]
static mut STATS: HashMap<StatsKey, StatsValue> = HashMap::with_max_entries(256, 0);

// 白名單 Map - 用於 bypass 特定 IP 的流量
#[map]
static mut BYPASS_MAP: HashMap<[u8; 4], u8> = HashMap::with_max_entries(65536, 0);

// 旁路控制標誌
#[map]
static mut BYPASS_ALL: u8 = 0;

#[repr(C)]
#[derive(Clone, Copy)]
struct StatsKey {
    cpu: u32,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct StatsValue {
    total_packets: u64,
    http_packets: u64,
    redirected_packets: u64,
    passed_packets: u64,
    bypassed_packets: u64,
}

/// 以太網 header 結構
#[repr(C)]
struct EthHdr {
    h_dest: [u8; 6],
    h_source: [u8; 6],
    h_proto: u16,
}

/// IP header 結構
#[repr(C)]
struct IpHdr {
    ihl: u8,
    version: u8,
    tos: u8,
    tot_len: u16,
    id: u16,
    frag_off: u16,
    ttl: u8,
    protocol: u8,
    check: u16,
    saddr: u32,
    daddr: u32,
}

/// TCP header 結構
#[repr(C)]
struct Tcphdr {
    source: u16,
    dest: u16,
    seq: u32,
    ack_seq: u32,
    res1: u16,
    doff: u16,
    window: u16,
    check: u16,
    urg_ptr: u16,
}

/// XDP 程序入口
/// 
/// 功能：
/// 1. 解析以太網/IP/TCP header
/// 2. 識別目標端口為 80 的 HTTP 流量
/// 3. 檢查白名單 bypass map
/// 4. 將 HTTP 流量重定向到 AF_XDP socket
/// 5. 非 HTTP 流量直接放行 (fastpath)
#[xdp]
pub fn vwire_xdp(ctx: XdpContext) -> u32 {
    unsafe {
        // 獲取 CPU ID 用於統計
        let cpu = bpf_get_smp_processor_id() as u32;
        let stats_key = StatsKey { cpu };
        
        // 更新總包數統計
        if let Some(stats) = STATS.get_ptr_mut(&stats_key) {
            (*stats).total_packets = (*stats).total_packets.wrapping_add(1);
        }

        // 檢查是否啟用全旁路模式 (watchdog 控制)
        let bypass_all_ptr = BYPASS_ALL as *mut u8;
        if *bypass_all_ptr != 0 {
            if let Some(stats) = STATS.get_ptr_mut(&stats_key) {
                (*stats).bypassed_packets = (*stats).bypassed_packets.wrapping_add(1);
            }
            return XDP_PASS;
        }

        // 確保至少有以太網 header (14 bytes)
        let data = ctx.data() as *const u8;
        let data_end = ctx.data_end() as *const u8;
        
        if data_end.offset_from(data) < 14 {
            return XDP_PASS;
        }

        // 解析以太網 header
        let eth = data as *const EthHdr;
        let h_proto = (*eth).h_proto.to_be();
        
        // 只處理 IPv4 流量
        if h_proto != ETH_P_IP {
            return XDP_PASS;
        }

        // 確保至少有 IP header (最小 20 bytes)
        let ip_start = data.add(14);
        if data_end.offset_from(ip_start) < 20 {
            return XDP_PASS;
        }

        // 解析 IP header
        let iph = ip_start as *const IpHdr;
        let ihl = (*iph).ihl as usize;
        let protocol = (*iph).protocol;
        let saddr = (*iph).saddr;
        
        // 檢查源 IP 是否在白名單中
        let saddr_bytes = saddr.to_ne_bytes();
        if BYPASS_MAP.get(&saddr_bytes).is_some() {
            if let Some(stats) = STATS.get_ptr_mut(&stats_key) {
                (*stats).bypassed_packets = (*stats).bypassed_packets.wrapping_add(1);
            }
            return XDP_PASS;
        }
        
        // 只處理 TCP 流量
        if protocol != IPPROTO_TCP {
            return XDP_PASS;
        }

        // 確保至少有 TCP header (最小 20 bytes)
        let tcp_start = ip_start.add(ihl * 4);
        if data_end.offset_from(tcp_start) < 20 {
            return XDP_PASS;
        }

        // 解析 TCP header
        let tcph = tcp_start as *const Tcphdr;
        let dest_port = (*tcph).dest.to_be();
        
        // 更新 HTTP 流量統計
        if dest_port == HTTP_PORT {
            if let Some(stats) = STATS.get_ptr_mut(&stats_key) {
                (*stats).http_packets = (*stats).http_packets.wrapping_add(1);
            }
            
            // 嘗試重定向到 AF_XDP socket
            let redirect_result = bpf_xdp_redirect_map(
                &mut XSCK_MAP as *mut _ as *mut _,
                0,
                XDP_REDIRECT as u64,
            );
            
            if redirect_result == 0 {
                // 重定向成功
                if let Some(stats) = STATS.get_ptr_mut(&stats_key) {
                    (*stats).redirected_packets = (*stats).redirected_packets.wrapping_add(1);
                }
                return XDP_REDIRECT;
            }
            
            // 重定向失敗，降級為 PASS
            return XDP_PASS;
        }

        // 非 HTTP 流量直接放行
        if let Some(stats) = STATS.get_ptr_mut(&stats_key) {
            (*stats).passed_packets = (*stats).passed_packets.wrapping_add(1);
        }
        XDP_PASS
    }
}

/// 獲取統計信息 (可選，用於調試)
#[no_mangle]
pub extern "C" fn get_stats(cpu: u32) -> *const StatsValue {
    unsafe {
        let key = StatsKey { cpu };
        match STATS.get_ptr(&key) {
            Some(stats) => stats,
            None => core::ptr::null(),
        }
    }
}

// 必要的 panic 處理
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    unsafe { core::hint::unreachable_unchecked() }
}
