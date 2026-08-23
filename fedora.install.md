## Installing on Fedora

##### 1. Download the RPM package from `[GitHub Releases]`

##### 2. Open a terminal in the folder where the package was downloaded

##### 3. Run the following command to prepare the system:

```bash
sudo dnf install -y \
  https://download1.rpmfusion.org/free/fedora/rpmfusion-free-release-$(rpm -E %fedora).noarch.rpm \
  https://download1.rpmfusion.org/nonfree/fedora/rpmfusion-nonfree-release-$(rpm -E %fedora).noarch.rpm && \
sudo dnf upgrade --refresh -y && \
sudo dnf config-manager setopt fedora-cisco-openh264.enabled=0
```

##### 4. Install the package:

```bash
sudo dnf install path/to/file/Formato*.rpm
```

> **Tip:** If you downloaded it to `~/Downloads`, run `sudo dnf install ~/Downloads/Formato*.rpm`

Done. Launch it from the app menu or by running `formato` in the terminal.
