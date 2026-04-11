---
name: fix_env
description: Kỹ năng xử lý kẹt Port và treo tiến trình trên Windows (Git Bash)
---

# Kỹ năng: Fix Environment (Windows Git Bash)

Kỹ năng này cung cấp cho Antigravity AI (bạn) các chiến lược cụ thể để áp dụng khi gặp vấn đề môi trường, đặc biệt là lỗi `EADDRINUSE` (kẹt port) hoặc treo tiến trình khi phát triển ứng dụng Node.js/Java trên **Windows** nhưng sử dụng Terminal là **Git Bash**.

## Vấn đề 1: Lỗi Kẹt Port (EADDRINUSE)
Khi server frontend (3000) hoặc backend (3001, 8080) không thể start lại do port đang bị chiếm giữ (thường là do server cũ crash hoặc terminal bị tắt ngang nhưng background process Node/Java vẫn chạy).

### Cách giải quyết (Pipeline Bash + Windows Native):
1. **Tìm & Diệt tiến trình (Tuyệt đối không dùng `lsof`):**
   Thay vì báo lỗi "lsof command not found", hãy sử dụng lệnh kết hợp `netstat` của Windows và các công cụ xử lý chuỗi của Bash:
   
   - **Kill Frontend (Port 3000):**
     ```bash
     netstat -ano | awk '/:3000/ {print $5}' | xargs -r tskill
     ```
   - **Kill Backend NestJS (Port 3001):**
     ```bash
     netstat -ano | awk '/:3001/ {print $5}' | xargs -r tskill
     ```
   - **Kill Backend Java (Port 8080):**
     ```bash
     netstat -ano | awk '/:8080/ {print $5}' | xargs -r tskill
     ```

2. **Xác nhận:** Sau khi chạy lệnh kill, bạn có thể tự tin gợi ý user khởi động lại server hoặc tự chạy background command khởi động server.

## Vấn đề 2: Lệnh cài đặt / chạy nền bị treo vĩnh viễn
Khi chạy các lệnh tải package nặng (`npm install`, `gradlew build`, `docker pull`) ở background và không thấy phản hồi sau 1-2 phút.

### Cách giải quyết:
1. **Giới hạn thời gian (Timeout):**
   Nếu bạn nghi ngờ một lệnh có rủi ro bị treo mạng (như tải từ server npm chậm), hãy luôn bọc nó bằng lệnh `timeout` của Bash:
   ```bash
   timeout 120 npm install
   ```
2. **Hủy ngang & Dọn dẹp:**
   Nếu đã lỡ chạy mà bị kẹt, hãy đề nghị user hủy lệnh bằng tay (Ctrl+C), xóa thư mục sinh ra dở dang (như `node_modules`), giải phóng port rồi chạy lại lệnh với `--prefer-offline` hoặc `--no-audit`.

## 🚨 Ghi nhớ cốt lõi
- **Hệ sinh thái:** Bạn đang sống trong sự giao thoa: File system là Windows, nhưng ngôn ngữ thao tác là Bash (POSIX).
- **Hành động:** Khi user phàn nàn "lệnh bị treo" hoặc "port lỗi", hãy **TỰ ĐỘNG THỰC THI** các pipeline `netstat | awk` ở trên để dọn đường thay vì yêu cầu user làm thủ công.
