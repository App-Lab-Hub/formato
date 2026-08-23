# Installing Formato on Linux (AppImage)

Formato is a cross-platform desktop app. This guide covers installation on Linux using the AppImage package.

---

## Dependencies

> **Note:** Formato uses LibreOffice for document-to-document conversion (e.g., PDF → DOCX, ODT → XLSX).
> **If LibreOffice is not installed, document conversion will not work.**

### Ubuntu / Debian Family

```bash
sudo apt update && sudo apt install libreoffice-writer libreoffice-calc -y
```

### Fedora Family

```bash
sudo dnf upgrade --refresh && sudo dnf install libreoffice-writer libreoffice-calc -y
```

---

## Launching the AppImage

### Default Launch (X11)

```bash
/path/to/Formato_1.0.0_amd64.AppImage
```

### Launch on Wayland (Fixes EGL Error)

On some systems (especially Wayland), you may encounter:
Could not create default EGL display: EGL_BAD_PARAMETER. Aborting...

To fix this, preload the system Wayland library:

```bash
LD_PRELOAD=$(ldconfig -p | grep "libwayland-client.so" | grep -E "x86-64|libc6,AArch64" | awk '{print $4}' | head -1)  /path/to/Formato_1.0.0_amd64.AppImage
```

> **Tip:** This command automatically finds the correct Wayland library on your system.

---

## Troubleshooting

| Issue                               | Solution                                                                 |
| ----------------------------------- | ------------------------------------------------------------------------ |
| **EGL_BAD_PARAMETER error**         | Launch with `LD_PRELOAD` as described above.                             |
| **Document conversion not working** | Install LibreOffice (`libreoffice-writer`, `libreoffice-calc`).          |
| **AppImage doesn't start**          | Make the file executable: `chmod +x Formato_1.0.0_amd64.AppImage`        |
| **AppImage blocked by system**      | Right-click → Properties → Permissions → Allow executing as program.     |
| **Flatpak alternative**             | Use the Flatpak version if available to avoid system library mismatches. |

---

## License

Formato is released under the [MIT License](LICENSE).
