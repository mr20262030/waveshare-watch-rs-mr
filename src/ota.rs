// OTA Boot Partition Switcher for Dual-Boot with Xiaozhi AI Assistant
// Manages the ESP32-S3 'otadata' partition at 0x00D000
// Layout:
//   ota_0 at 0x020000: Xiaozhi AI Assistant (official binary)
//   ota_1 at 0x410000: Rust Watch Firmware
//
// Sequence logic:
//   Sector 0 (0x00D000): seq=1, state=VALID (Xiaozhi, fallback)
//   Sector 1 (0x00E000): seq=2, state=VALID (Rust, active default)
//
// When jumping to Xiaozhi:
//   Sector 0 (0x00D000) is written with seq=3, state=PENDING_VERIFY.
//   Bootloader selects seq=3 (ota_0 = Xiaozhi).
//   On physical RESET, bootloader auto-rolls back to seq=2 (ota_1 = Rust).
//   On Rust startup, ensure_default_rust_boot restores seq=1 and seq=2.

use embedded_storage::nor_flash::{NorFlash, ReadNorFlash};
use esp_storage::FlashStorage;

const OTADATA_SECTOR_0: u32 = 0x00D000;
const OTADATA_SECTOR_1: u32 = 0x00E000;
const SECTOR_SIZE: u32 = 4096;

const ESP_OTA_IMG_PENDING_VERIFY: u32 = 1;
const ESP_OTA_IMG_VALID: u32 = 2;

// IEEE 802.3 CRC32 implementation for no_std
fn crc32_ieee(data: &[u8]) -> u32 {
    let mut crc = 0xFFFFFFFFu32;
    for &byte in data {
        crc ^= byte as u32;
        for _ in 0..8 {
            if crc & 1 != 0 {
                crc = (crc >> 1) ^ 0xEDB88320;
            } else {
                crc >>= 1;
            }
        }
    }
    !crc
}

fn make_entry(seq: u32, state: u32) -> [u8; 32] {
    let mut entry = [0xFFu8; 32];
    entry[0..4].copy_from_slice(&seq.to_le_bytes());
    entry[24..28].copy_from_slice(&state.to_le_bytes());
    let crc = crc32_ieee(&entry[0..28]);
    entry[28..32].copy_from_slice(&crc.to_le_bytes());
    entry
}

/// Sets the boot partition to ota_0 (Xiaozhi) with PENDING_VERIFY state (seq=3).
/// When user presses physical RESET, ESP-IDF bootloader automatically rolls back
/// to ota_1 (Rust) because Xiaozhi never cancels rollback.
pub fn switch_to_xiaozhi_and_reboot() -> ! {
    let mut storage = FlashStorage::new();

    let entry = make_entry(3, ESP_OTA_IMG_PENDING_VERIFY);
    let _ = storage.erase(OTADATA_SECTOR_0, OTADATA_SECTOR_0 + SECTOR_SIZE);
    let _ = storage.write(OTADATA_SECTOR_0, &entry);

    esp_hal::system::software_reset();
}

/// Called on normal Rust boot to ensure otadata always defaults to Rust (ota_1, seq=2).
/// Restores clean state (Sector 0: seq=1, Sector 1: seq=2).
pub fn ensure_default_rust_boot() {
    let mut storage = FlashStorage::new();

    let mut sec0_bytes = [0u8; 4];
    let mut sec1_bytes = [0u8; 4];
    let r0 = storage.read(OTADATA_SECTOR_0, &mut sec0_bytes);
    let r1 = storage.read(OTADATA_SECTOR_1, &mut sec1_bytes);

    let sec0_seq = if r0.is_ok() { u32::from_le_bytes(sec0_bytes) } else { 0 };
    let sec1_seq = if r1.is_ok() { u32::from_le_bytes(sec1_bytes) } else { 0 };

    // If otadata is not in the normal default state (sec0=1, sec1=2), restore it
    if sec0_seq != 1 || sec1_seq != 2 {
        let entry0 = make_entry(1, ESP_OTA_IMG_VALID);
        let entry1 = make_entry(2, ESP_OTA_IMG_VALID);

        let _ = storage.erase(OTADATA_SECTOR_0, OTADATA_SECTOR_0 + SECTOR_SIZE);
        let _ = storage.write(OTADATA_SECTOR_0, &entry0);

        let _ = storage.erase(OTADATA_SECTOR_1, OTADATA_SECTOR_1 + SECTOR_SIZE);
        let _ = storage.write(OTADATA_SECTOR_1, &entry1);
    }
}
