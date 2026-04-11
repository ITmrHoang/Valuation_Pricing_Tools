---
description: Pipeline flow BA -> DEV -> QC làm việc độc lập cho tính năng mới
---

# 🚀 QUY TRÌNH BA -> DEV -> QC (PIPELINE-FLOW)

Quy trình chuẩn cho một Module (tính năng lớn).

## Giai đoạn 1: Lên Kế Hoạch (BA)
- Dựa trên `requirements.md` (giữ nguyên lịch sử, không xóa yêu cầu cũ). Cập nhật phiên bản hoặc thêm notes. **Lưu ý:** Chỉ cập nhật khi đó là tính năng lớn/nghiệp vụ cốt lõi, KHÔNG ghi log sửa lỗi vặt hay đổi UI nhỏ.
- Tham khảo các patterns có sẵn, phân tích thiết kế cập nhật vào `analyst.md` để đồng bộ. `analyst.md` là bộ nhớ kiến trúc dài hạn, tuyệt đối không ghi quá trình fix bug hay sửa prompt.
- Phân tách ra thành danh sách công việc (Tasks) trên `tasks.md`. Vẫn chưa đụng vào Code.

## Giai đoạn 2: Phát Triển (DEV - Code)
- Liên tục dựa vào file `tmp/task_processing_<chat_id>.md` hoặc nhìn `tasks.md` để triển khai viết code.
- DEV tập trung 100% vào việc rẽ nhánh code các module. Unit-test sẽ được thực hiện khi viết logic nội tại thay vì test qua giao diện. DEV không nên mất quá trình để thực hiện Visual tests khi code.
- Nếu chunk logic nhỏ hoàn thành đánh dấu trên tmp file nhưng `tasks.md` giữ lại đến khi toàn bộ logic module viết xong.

## Giai đoạn 3: Kiểm soát chất lượng (QC - Browser)
- Lúc này chạy trình duyệt mở localhost / app. Kiểm thử toàn bộ User Flow đã làm.
- Quay video hoặc chụp Screenshots với Browser Subagent để ghi lại minh chứng nếu tìm thấy bugs.
- Cập nhật bugs thẳng vô lại `tasks.md` để DEV tiếp tục khắc phục. Vòng lặp này lặp lại cho tới khi chạy luồng trơn tru.

## Cuối Chu Kỳ
- Module được duyệt hoàn tất, `tasks.md` cập nhật tiến trình `[x]`. File `walkthrough.md` được render cho user xem.
- Hỏi người dùng xem có muốn lưu lại các file Test Media (Video/Image) đã tạo trong ổ cứng `artifacts/` hay `.gemini/antigravity` không. Nếu user chẳng quan tâm hoặc "Okay" hoặc cho qua => Lệnh `rm` hoặc `del` xóa toàn phần Media để hoàn tất và nhẹ ổ đĩa. 
- **QUAN TRỌNG:** Phải BẮT BUỘC dọn dẹp (xóa) cả các file rác tạm thời như file kết quả terminal (`*.txt`, `*.log`), các script nội bộ sinh ra để test hoặc debug, để giữ workspace luôn sạch sẽ. File báo cáo chính `*.md` thì giữ nguyên trạng, bộ lưu chunking được giữ đến lúc thay đổi task module mới hoàn toàn.
