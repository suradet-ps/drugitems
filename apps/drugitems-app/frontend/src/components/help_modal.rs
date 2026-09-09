//! Help modal - what the app does, how to use it, and its guarantees.

use leptos::ev;
use leptos::prelude::*;

use crate::components::icons::IconX;
use crate::state::AppState;

#[component]
pub fn HelpModal(state: AppState) -> impl IntoView {
    let close = move || state.help_open.set(false);
    let open_flag = state.help_open;
    let close_on_escape = move |event: ev::KeyboardEvent| {
        if event.key() == "Escape" && open_flag.get_untracked() {
            close();
        }
    };
    let escape_handle = window_event_listener(ev::keydown, close_on_escape);
    let _escape_handle = StoredValue::new(escape_handle);

    view! {
        <div
            class="modal-backdrop"
            style:display=move || {
                if state.help_open.get() { "flex" } else { "none" }
            }
            on:click=move |_| close()
        >
            <section class="modal" on:click=move |ev| ev.stop_propagation()>
                <h2 class="modal__title">"วิธีใช้ DrugItems"</h2>
                <div class="help-body">
                    <h4>"โปรแกรมนี้ทำอะไร"</h4>
                    <p>
                        "ตรวจสอบว่าข้อมูลในตารางยา (drugitems) ของฐานข้อมูล MySQL ยังตรงกับไฟล์ snapshot ที่เคยส่งออกไว้หรือไม่ - ถ้าไม่ตรง แสดงว่ามีใครบางคน (หรือบางโปรแกรม) แก้ไขข้อมูลในฐานข้อมูล"
                    </p>

                    <h4>"ขั้นตอน"</h4>
                    <ol>
                        <li>"ตั้งค่าการเชื่อมต่อ MySQL (แนะนำบัญชีที่อ่านได้อย่างเดียว) แล้วกด ทดสอบ"</li>
                        <li>"เลือกไฟล์ snapshot - ไฟล์ Excel ที่ส่งออกจากตารางยาในอดีต (ทุกคอลัมน์ ทุกค่าในไฟล์นี้คือข้อมูลที่เชื่อถือได้)"</li>
                        <li>"กด เปรียบเทียบข้อมูล - โปรแกรมอ่านตารางจากฐานข้อมูล (SELECT เท่านั้น) แล้วเปรียบเทียบทีละรายการ ทีละคอลัมน์"</li>
                        <li>"ดูผลในช่องขวา: รายการที่ต่าง / ใหม่ / หาย และรายละเอียดค่าเก่า-ใหม่ของแต่ละคอลัมน์"</li>
                        <li>"กด บันทึกรายงาน CSV เพื่อเก็บหลักฐาน"</li>
                    </ol>

                    <h4>"การรับประกัน"</h4>
                    <ul>
                        <li>"อ่านอย่างเดียว: ทุกคำสั่งที่ส่งไปยังฐานข้อมูลต้องขึ้นต้นด้วย SELECT/SHOW/DESCRIBE/EXPLAIN เท่านั้น - ไม่มีการแก้ไขข้อมูล"</li>
                        <li>"รหัสผ่านถูกเข้ารหัส (AES-256-GCM) โดยกุญแจอยู่ใน Windows Credential Manager"</li>
                        <li>"ค่า NULL กับช่องว่างในไฟล์ถือว่าเหมือนกัน (ไฟล์ Excel แยกสองแบบนี้ไม่ออก)"</li>
                    </ul>

                    <h4>"ข้อจำกัดที่ทราบ"</h4>
                    <ul>
                        <li>"เลขที่เขียนเป็นตัวหนังสือที่ดูเหมือนตัวเลข (เช่น 010 กับ 10) อาจถูกมองว่าเหมือนกัน"</li>
                        <li>"คอลัมน์ที่มีในฐานข้อมูลแต่ไม่มีในไฟล์ (หรือกลับกัน) จะถูกข้ามการเปรียบเทียบและแสดงเป็นหมายเหตุ"</li>
                    </ul>
                </div>

                <div class="modal__actions">
                    <button class="button-secondary button-secondary--inline" on:click=move |_| close()>
                        <IconX class="icon" />
                        "ปิด"
                    </button>
                </div>
            </section>
        </div>
    }
}
