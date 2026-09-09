# DrugItems

โปรแกรมเดสก์ท็อปตรวจสอบว่า **ข้อมูลในตารางยา (`drugitems`) ของฐานข้อมูล
MySQL ยังตรงกับไฟล์ snapshot ที่เคยส่งออกไว้หรือไม่** — อ่านอย่างเดียว
(อ่านอย่างเดียว) ไม่มีการแก้ไขข้อมูลใดๆ

ไฟล์ snapshot (`drugiterms-20260908.xls`: 657 รายการ × 202 คอลัมน์) คือ
**source of truth** — ทุกคอลัมน์ ทุกค่าในไฟล์คือสิ่งที่ควรเป็น ถ้าฐานข้อมูล
ต่างจากไฟล์ แสดงว่ามีการแก้ไขข้อมูล

> สร้างโดยปรับโครงสร้างจากโปรเจกต์ [Med Recon](https://github.com/suradet-ps/med-recon)
> (Tauri 2 + Leptos 0.8) · ดูรายละเอียดการออกแบบที่ `docs/DESIGN.md`

## คุณสมบัติ

- ตั้งค่าการเชื่อมต่อ MySQL (รหัสผ่านเข้ารหัส AES-256-GCM + Windows Credential Manager)
- เลือกไฟล์ snapshot `.xls/.xlsx` (ผ่าน native dialog)
- เปรียบเทียบทั้งตารางทีละรายการ ทีละคอลัมน์: รายการที่ **ต่าง / ใหม่ใน DB / หายจาก DB**
- Dashboard: verdict + จำนวนสรุป + คอลัมน์ที่ถูกแก้บ่อย + กรอง/ค้นหา + ขยายดูค่าเก่า-ใหม่
- บันทึกรายงานความต่างเป็น CSV
- Read-only บังคับในโค้ด (ทุกคำสั่งขึ้นต้นด้วย SELECT/SHOW/DESCRIBE/EXPLAIN เท่านั้น)

## เริ่มต้นใช้งาน

```bash
rustup target add wasm32-unknown-unknown
cargo install trunk

cargo tauri dev      # โหมดพัฒนา
cargo tauri build    # สร้างตัวติดตั้ง
```

ตรวจสอบโค้ด/ทดสอบ (ไม่ต้องมีฐานข้อมูล):

```bash
cargo check -p drugitems-core -p drugitems-snapshot -p drugitems-db \
            -p drugitems-config -p drugitems-bridge -p drugitems-app
cargo check -p drugitems-frontend --target wasm32-unknown-unknown
cargo test -p drugitems-core -p drugitems-db -p drugitems-app -p drugitems-config
cargo run -p drugitems-snapshot --example parse -- drugiterms-20260908.xls
```

## โครงสร้าง

```
crates/
  drugitems-core/      # โมเดล + กลไกเปรียบเทียบ (canonicalization, วันที่ Excel)
  drugitems-snapshot/  # อ่านไฟล์ .xls/.xlsx (calamine)
  drugitems-db/        # sqlx MySQL read-only (schema + อ่านตาราง)
  drugitems-config/    # connection เข้ารหัส + settings.json
  drugitems-bridge/    # invoke ข้าม IPC ฝั่ง WASM
apps/
  drugitems-app/       # shell Tauri 2 + frontend Leptos 0.8
```

## หมายเหตุ

- ไอคอนแอปเป็นไอคอนชั่วคราว (copy มาจาก Med Recon) — ยังต้องออกแบบใหม่
- ดูเอกสาร: [`DESIGN.md`](DESIGN.md) (ระบบการออกแบบ UI — สี/ฟอนต์/ส่วนประกอบ),
  [`docs/DESIGN.md`](docs/DESIGN.md) (สำเนาชุดเดียวกัน), [`AGENTS.md`](AGENTS.md) (ข้อกำหนดสำหรับนักพัฒนา)

Licensed under MIT OR Apache-2.0.
