---
description: "TypeScript and Vue 3 Strict Coding Standards"
---

# 🛠️ Rule: Strict TypeScript & Vue 3 Composition API

## 🎯 Mục tiêu
Quy tắc này hướng dẫn Antigravity (AI) tuân thủ nghiêm ngặt các tiêu chuẩn của TypeScript và Vue 3 (Composition API) để tránh các lỗi type, lỗi compile và lỗi runtime liên quan đến dự án Nuxt 3 / Vue 3.

## 📜 Quy tắc thực thi (BẮT BUỘC DÀNH CHO AI)

### 1. Vue 3 Composition API
- Mọi file `.vue` tạo mới hoặc chỉnh sửa phải dùng Composition API với `<script setup lang="ts">`.
- **TUYỆT ĐỐI KHÔNG** dùng Options API (như `data()`, `methods: {}`, `mounted()`).
- Các hàm lifecycles phải dùng Composition hook (`onMounted`, `onUnmounted`...).

### 2. Strict Type Declaration (Định nghĩa Type Rõ Ràng)
- Khi khai báo biến trạng thái, BẮT BUỘC phải gán kiểu (Generic Type). 
  * ✅ Đúng: `const items = ref<User[]>([])` 
  * ❌ Sai: `const items = ref([])` (dẫn đến kiểu `never[]`).
- Thay vì dùng `any` bừa bãi, hãy tốn thêm công khai báo `interface` hoặc `type` cho dữ liệu.
- Nếu gán Object cho một Type có sẵn, không được viết thừa (hoặc sai tên) bất kỳ thuộc tính nào không có mặt trong Type/Interface đó (Lỗi *Object literal may only specify known properties*).

### 3. Khai báo Props và Emits
- Dùng `defineProps` với Typescript:
  ```ts
  const props = defineProps<{
    title: string;
    items?: string[];
  }>()
  ```
- Dùng `defineEmits` với Typescript:
  ```ts
  const emit = defineEmits<{
    (e: 'update', id: number): void
  }>()
  ```

### 4. Nuxt 3 Imports
- Nuxt 3 tự động import các Reactivity API (`ref`, `computed`, `watch`, `onMounted`, ...). Tuyệt đối **KHÔNG** ghi lệnh `import { ref } from 'vue'` trên đầu file, trừ khi Linter yêu cầu.
- Khi cần import chỉ Type từ file khác, hãy sử dụng `import type { ... } from '...'` để tối ưu quá trình build của Vite.

### 5. Nuxt Data Fetching
- Dùng `useFetch` hoặc `useAsyncData` có truyền Type trả về: `const { data, pending, error } = await useFetch<ApiResponse>('/api/...')`
- Trong Template `.vue`, giao diện bắt buộc phải có xử lý `v-if="pending"` (đang tải) và `v-if="error"` (báo thẻ hiển thị lỗi) khi fetching dữ liệu.

### 6. Nil/Undefined Safety Checks
- TS Strict mode sẽ bắt lỗi truy xuất properties từ Object có thể `null`/`undefined`. 
- Bắt buộc Dùng **Optional Chaining** (`obj?.property`) và **Nullish Coalescing** (`value ?? fallback`) trước khi truy xuất giá trị từ API trả về.
