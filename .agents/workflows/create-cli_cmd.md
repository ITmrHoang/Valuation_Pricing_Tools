---
description: create and process command
---

# 🛠️ Rule: Antigravity Autonomous CLI Execution (Windows CMD)

## 🎯 Mục tiêu
Quy tắc này hướng dẫn **chính Antigravity (AI)** cách tự động chạy lệnh (thông qua tool `run_command`) một cách tối ưu, an toàn và không bị lỗi trong môi trường **Command Prompt (CMD) trên hệ điều hành Windows**.

## 📜 Quy tắc thực thi lệnh (Dành cho AI)
1. **Nhận thức Môi trường (Environment Awareness):**
   - **OS:** Windows.
   - **Shell:** CMD (Command Prompt).
   - **CẢNH BÁO:** Bạn **KHÔNG ĐƯỢC PHÉP** sử dụng các lệnh POSIX chuẩn của Linux (`ls`, `rm`, `cp`, `mv`, `grep`, `awk`, `sed`, `cat`, `curl`, `find`) vì chúng sẽ gây lỗi `command not found` trên CMD thuần.

2. **Cách xử lý khi thiếu lệnh Linux (Dùng Windows Native):**
   - Thay vì dùng `ls`, BẮT BUỘC dùng `dir`.
   - Thay vì dùng `cat`, BẮT BUỘC dùng `type`.
   - Thay vì dùng `rm -rf`, BẮT BUỘC dùng `rmdir /s /q` (cho thư mục) và `del /f /q` (cho file).
   - Thay vì dùng `grep`, BẮT BUỘC dùng `findstr`.
   - Khi cần diệt process/tìm port để dọn dẹp môi trường:
     Dùng vòng lặp FOR của CMD thay cho pipeline: `FOR /F "tokens=5" %T IN ('netstat -a -n -o ^| findstr :3000') DO taskkill /F /PID %T`

3. **Cú pháp Đường dẫn (Path Handling):**
   - Trong CMD, bắt buộc dùng dấu gạch chéo ngược `\` cho đường dẫn thực thi lệnh native (`D:\Project\e-lerning`).
   - Hãy chắc chắn bao quanh đường dẫn có khoảng trắng bằng cặp dấu ngoặc kép `" "`. (Ví dụ: `"D:\My Project\test.txt"`).

4. **Xử lý treo Terminal (Process Hanging) - KHẨN CẤP:**
   - Khi chạy lệnh mạng dễ kẹt mạng/bộ nhớ trên Windows, cân nhắc giới hạn tiến trình bằng cách bọc script hoặc báo cho user.
   - Nếu bạn gọi `npm run dev`, hãy đảm bảo dùng tuỳ chọn background hoặc dùng `WaitMsBeforeAsync` vì nó sẽ chiếm dụng Process.

   ### 4.1 Quy trình xử lý Background Command bị Treo (MANDATORY)
   Khi gọi `run_command` và sau đó dùng `command_status` để kiểm tra kết quả, NẾU phát hiện lệnh **không có output mới sau 2 lần gọi `command_status`** liên tiếp (mỗi lần chờ ≥ 30 giây), thì BẮT BUỘC phải thực hiện theo trình tự sau:

   > ⚠️ **NGOẠI LỆ — KHÔNG ÁP DỤNG** cho các lệnh chạy liên tục (long-running/persistent process). Các lệnh sau đây **KHÔNG BAO GIỜ** bị coi là "treo" và **KHÔNG ĐƯỢC Kill PID**:
   > - `npm run dev` / `npx nuxi dev` / `nuxt dev` (Dev server)
   > - `npm start` / `node server.js` (Production server)
   > - `npm run preview` (Preview server)
   > - Bất kỳ lệnh nào mở một HTTP server hoặc file watcher chạy liên tục
   >
   > Với những lệnh trên, chỉ **Kill PID SAU KHI ĐÃ XONG TOÀN BỘ CÔNG VIỆC** (user xác nhận hoàn thành hoặc AI kết thúc task cuối cùng), hoặc khi user chủ động yêu cầu tắt.

   **Bước 1: Xác nhận treo** — Gọi `command_status` lần thứ 3 với `WaitDurationSeconds: 30`. Nếu vẫn không có output delta mới → xác nhận lệnh bị TREO.

   **Bước 2: Kill PID cũ** — Gọi `send_command_input` với `Terminate: true` lên CommandId bị treo để giải phóng process.

   **Bước 3: Chuyển phương án thay thế** — Theo thứ tự ưu tiên:
   1. **Dùng NodeJS script inline** (`node -e "..."`) thay cho lệnh CMD gốc bị treo (ưu tiên cao nhất vì NodeJS ổn định trên cả CMD và Bash).
   2. **Dùng tool native của AI** (`list_dir`, `find_by_name`, `view_file`, `write_to_file`...) nếu mục đích ban đầu là thao tác file/thư mục.
   3. **Báo cho user chạy tay** — Nếu cả 2 phương án trên đều không khả thi, cung cấp lệnh chính xác trong tin nhắn để user tự paste vào Terminal.

   **Bước 4: Ghi nhận** — Ghi chú lại trong `TaskSummary` rằng lệnh CMD bị treo và đã chuyển sang phương án thay thế, để tránh lặp lại lỗi tương tự.

   ### 4.2 Phòng tránh treo từ đầu (Preventive)
   - Với các lệnh copy/move file có đường dẫn DÀI hoặc chứa KÝ TỰ ĐẶC BIỆT (dấu chấm `.`, gạch dưới `_`, số dài), **ưu tiên dùng `node -e`** ngay từ đầu thay vì `copy` / `xcopy` của CMD.
   - Ví dụ chuẩn:
     ```cmd
     node -e "const fs=require('fs'); fs.copyFileSync('source.png','dest.png'); console.log('Done');"
     ```
   - Với các lệnh `dir` (liệt kê thư mục), **ưu tiên dùng tool `list_dir`** của AI thay vì gọi CMD vì tool native không bao giờ bị treo.

5. **Thao tác File Code:**
   - Dù CMD có lệnh `echo` hay `type`, **TUYỆT ĐỐI HẠN CHẾ** dùng CLI để sửa code. 
   - Thay vào đó, ưu tiên sử dụng các tool native của AI như `replace_file_content` hoặc `multi_replace_file_content` để đảm bảo độ chính xác.

## 🚫 Lệnh tuyệt đối KHUYÊN TRÁNH khi AI tự chạy trên CMD
- Tuyệt đối không gọi các lệnh của PowerShell (như `Get-Content`, `Remove-Item`) trong CMD trừ khi bạn bọc nó trong `powershell -c "..."`.
- Không gọi lệnh tắt ngang `cd` không có cờ `/d` trên Windows. Muốn chuyển ổ đĩa qua CMD, phải dùng `cd /d D:\Folder`. Tuy nhiên với tool `run_command`, hãy dùng đối số `Cwd` trực tiếp.

## 🧹 6. Dọn Dẹp Tài Nguyên (Cleanup & Cancel) - BẮT BUỘC
- **TUYỆT ĐỐI KHÔNG ĐỂ QUÊN** các Terminal chạy ngầm.
- Ngoại trừ các tiến trình Server (`npm run dev`, `python main.py`...), đối với các lệnh thông thường hoặc lệnh test bị kẹt, sau khi đã gọi `command_status` lấy đủ thông tin và xong việc, BẮT BUỘC phải gọi tool `send_command_input` với tham số `Terminate: true` để đóng/tắt CommandId đó, giải phóng bộ nhớ và tránh bị rác Terminal cho hệ thống của user!