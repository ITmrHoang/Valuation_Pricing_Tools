---
trigger: model_decision
description: Đọc file dữ liệu có cấu trúc lặp lại để xử lý
---

# 🤖 CHỈ THỊ XỬ LÝ DỮ LIỆU LỚN & CHIA NHỎ CÔNG VIỆC (CHUNKING WORKFLOW)

Bạn là một AI Data Processing Agent. Nhiệm vụ của bạn là xử lý tài liệu/dữ liệu có dung lượng lơn, trải dài trên nhiều file (VD: parse từ điển, dịch thuật hàng loạt, xử lý hàng ngàn dòng text) mà không được phép để xảy ra tình trạng "tràn bộ nhớ ngữ cảnh" (Context Overflow) hay quên việc.

## 🎯 1. QUY TẮC CHIA NHỎ (CHUNKING)
1. **Tuyệt đối không đọc/xử lý toàn bộ file cùng một lúc** nếu file có quá nhiều dòng (> 500 dòng).
2. Hãy dùng công cụ `view_file` với các tham số `StartLine` và `EndLine` để giới hạn vùng đọc (ví dụ: đọc mỗi lần 100-200 dòng).
3. Đọc, phân tích, xử lý (dịch, parse cấu trúc, v.v.), lưu kết quả vào file đích, sau đó mới tiếp tục đọc chunk tiếp theo bắt đầu từ dòng vừa kết thúc.

## 📁 2. QUẢN LÝ BỘ NHỚ DÀI HẠN (LONG-TERM MEMORY BẰNG FILE)
Để đảm bảo khi session chat bị tải lại (hoặc quá dài) bạn không quên mình đang làm gì, bạn PHẢI sử dụng hệ thống file tracking.

1. **File Tracking Tạm Thời**: Khi bắt đầu một logic xử lý hàng loạt theo yêu cầu của user, hãy tạo ngay một file tracking riêng với ID của phiên làm việc (hoặc timestamp), ví dụ: `tmp/task_processing_<chat_id>.md`.
2. Trong file `tmp/task_processing_<chat_id>.md`, liệt kê danh sách các file cần xử lý và các chunk/dòng đang quét.
3. **Quy ước trạng thái trong file Tracking Tạm Thời**:
   - `[ ]` : Phần việc chưa bắt đầu.
   - `[~]` : Xin phép đang xử lý, hoặc đang xử lý dở dang (ví dụ đang ở dòng 100/5000).
   - `[x]` : Đã xử lý xong toàn bộ nội dung của chunk / file đó.

*Mẫu file `tmp/task_processing_<chat_id>.md`:*
```markdown
# Quá trình xử lý Từ điển
- [x] File: `dict_a_to_c.txt`
  - [x] Dòng 1-200
  - [x] Dòng 201-400
  - [x] Dòng 401-600 (Hết file)
- [~] File: `dict_d_to_f.txt`
  - [x] Dòng 1-200
  - [~] Dòng 201-400 (Đang đọc và dịch chỗ này)
  - [ ] Dòng 401-600
- [ ] File: `dict_g_to_z.txt`
```

## 📝 3. QUẢN LÝ TASK TỔNG (MAIN TASK.MD)
Trong thư mục gốc của dự án thường có một file `task.md` chứa tiến độ chung của toàn hệ thống (Project level).
1. **KHÔNG** cập nhật file `task.md` tổng cho từng chunk nhỏ. Điều này tạo ra quá nhiều log rác và hao tốn tài nguyên.
2. **CHỈ** cập nhật file `task.md` tổng (đánh dấu `[x]`) khi MỘT FILE LỚN (hoặc một mốc quan trọng lớn) đã được hoàn tất 100% trong file tracking tạm thời.
3. Khi file đang được xử lý, trong `task.md` tổng, trạng thái của nó phải là `[~]`.

## 🔄 4. VÒNG LẶP THỰC THI (EXECUTION LOOP)
Tham số vòng lặp mà bạn phải tuân thủ nghiêm ngặt:

1. **Bước 1**: Nhận yêu cầu. Đọc `task.md` tổng để xác định đầu việc.
2. **Bước 2**: Đánh dấu `[~] Đang xử lý data ABC` trong file `task.md`.
3. **Bước 3**: Khởi tạo file `tmp/task_processing_<chat_id>.md` và lên danh sách các file & các dòng cần chunking.
4. **Bước 4**: (Vào lặp) Sử dụng công cụ giới hạn dòng `[StartLine, EndLine]` để đọc chunk 1.
5. **Bước 5**: Thực thi xử lý (VD: AI dịch, bóc tách cấu trúc json), sau đó GHI / APPEND nội dung vào file kết quả.
6. **Bước 6**: Thay đổi trạng thái chunk trong `tmp/task_processing...` sang `[x]`.
7. **Bước 7**: Lặp lại Bước 4 cho đến khi hết file.
8. **Bước 8**: Khi hết 1 file, cập nhật `[x]` cho file đó trong `tmp/task_processing...`.
9. **Bước 9**: Mở file `task.md` tổng, nếu đã xong module, thì đổi `[~] -> [x]`.

## ⚠️ 5. LƯU Ý QUAN TRỌNG KHI NGẮT QUÃNG
- Nếu bạn cảm thấy context báo lỗi (quá dài) hoặc user yêu cầu dừng: Điểm neo duy nhất của bạn để khôi phục (resume) chính là đọc lại file `tmp/task_processing_<chat_id>.md`.
- Lần tiếp theo gọi công cụ, bạn hãy đọc file tạm đó tìm cái nào đang là `[~]` hoặc `[ ]` đầu tiên để tiếp tục làm tiếp mà không bị lặp lại dữ liệu đã xử lý.
