# Folder Cleaner

Ứng dụng GUI viết bằng Rust giúp dọn dẹp, phân loại và tìm file trùng lặp. Giao diện song ngữ **Tiếng Việt / English**.

---

## Tính năng

| Tính năng | Mô tả |
|-----------|-------|
| 🧹 **Dọn dẹp** | Quét thư mục, xem cây file, chọn & xóa file cũ an toàn |
| 📂 **Sắp xếp** | Tự động phân loại file vào Documents/, Images/, Videos/… |
| 🔍 **Trùng lặp** | Tìm file trùng bằng SHA-256 hash, xóa hàng loạt |
| ⚙ **Cài đặt** | Dark/Light/System theme · Tiếng Việt/English |
| 🗑 **An toàn** | Xóa vào Recycle Bin, không bao giờ xóa vĩnh viễn |

---

## Giao diện

```
┌─────┬──────────────────────────────────────────────┐
│  🧹 │  🧹 Công Cụ Dọn Dẹp                         │
│  🔍 │  Thư mục: C:\Users\...\Downloads  [Đổi]     │
│  ⚙  │  [Quét lại] [Sắp xếp] [Chọn file cũ] [Xóa] │
│     ├──────────────────────────────────────────────│
│     │  ☐  Tên            Ngày tạo  Ngày sửa  Size │
│     │  ☑ 📄 report.pdf   2024-01   2024-01    2MB │
│     │  ☐ 📦 archive.zip  2023-12   2024-01   50MB │
│     ├──────────────────────────────────────────────│
│     │  Đã chọn: 1 file (2MB) │ Tổng: 156 file     │
└─────┴──────────────────────────────────────────────┘
```

---

## Yêu cầu

- **OS:** Windows 10/11 (font Segoe UI + Segoe UI Emoji)
- **Rust:** edition 2024 (nightly hoặc stable ≥ 1.85)

---

## Build & Chạy

```powershell
# Debug (development)
cargo run

# Release (tối ưu, binary nhỏ)
cargo build --release
# Binary: target/release/folder_cleaner.exe
```

---

## Cấu trúc thư mục

```
src/
├── main.rs              # Entry point (960×640)
├── app.rs               # App struct, font/style setup, routing
├── scanner.rs           # Quét thư mục đệ quy
├── file_info.rs         # FileEntry, FileCategory, SortState
├── lang.rs              # Đa ngôn ngữ: Language enum + Lang struct (VI/EN)
├── utils.rs             # format_size(), format_date()
├── ui/
│   ├── colors.rs        # Màu sắc theo dark/light mode
│   ├── theme.rs         # Design tokens tập trung (Theme::DEFAULT)
│   ├── components/
│   │   ├── sidebar.rs   # Điều hướng 3 trang
│   │   ├── toolbar.rs   # Rescan, Sort, SelectOld, Delete
│   │   ├── tree_view.rs # Cây file có sort, expand, checkbox
│   │   └── dialogs.rs   # ConfirmDelete, ConfirmSort, Progress
│   └── pages/
│       ├── cleanup.rs         # Trang dọn dẹp
│       ├── duplicate_finder.rs# Trang tìm trùng
│       └── settings.rs        # Cài đặt theme & ngôn ngữ
└── actions/
    ├── cleaner.rs       # Xóa file (background thread + progress)
    └── sorter.rs        # Phân loại file vào thư mục con
```

---

## Cấu hình

File `settings.ini` tự động tạo cạnh `.exe`:

```ini
theme=System        # System | Light | Dark
language=Vietnamese # Vietnamese | English
```

---

## Dependencies chính

| Crate | Phiên bản | Mục đích |
|-------|----------|---------|
| `eframe` | 0.33.3 | GUI framework |
| `trash` | 5 | Xóa vào Recycle Bin |
| `rfd` | 0.17.2 | Native folder picker |
| `sha2` + `hex` | 0.10.9 / 0.4.3 | Hash cho duplicate finder |
| `dirs` | 6 | Đường dẫn Downloads |
| `open` | 5 | Mở file bằng app mặc định |

---

## Tài liệu chi tiết

Xem [`docs/application_design.md`](docs/application_design.md) để biết kiến trúc đầy đủ, cấu trúc dữ liệu, design token system và luồng xử lý.
