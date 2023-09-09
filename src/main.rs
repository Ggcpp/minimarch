use inquire::{Confirm, MultiSelect, Select, Text};
use serde_json::Value;
use std::{fs::File, io::Write, process::Command};

fn main() {
    // Internet
    let ans = Select::new(
        "Are you connected to internet?",
        vec!["yes", "I will check"],
    )
    .prompt()
    .unwrap();

    if ans == "I will check" {
        println!("type 'minimarch help internet' if you need help");
        return;
    }

    // Check efivars
    Command::new("ls")
        .arg("/sys/firmware/efi/efivars")
        .output()
        .expect("failed to read efivars");

    // ask drive to partition
    let output = Command::new("lsblk")
        .arg("-o")
        .arg("NAME,SIZE,MOUNTPOINTS")
        .output()
        .unwrap();

    let json = Command::new("lsblk").arg("-J").output().unwrap();
    let json: Value = serde_json::from_str(&String::from_utf8(json.stdout).unwrap()).unwrap();

    let mut options: Vec<&str> = Vec::new();
    for device in json["blockdevices"].as_array().unwrap() {
        options.push(device["name"].as_str().unwrap());
    }

    let partition = Select::new("Drive to partition:", options)
        .with_help_message(&String::from_utf8(output.stdout).unwrap())
        .prompt()
        .unwrap();

    // cfdisk
    loop {
        Command::new("cfdisk")
            .arg(format!("/dev/{partition}"))
            .status()
            .unwrap();

        let ans = Confirm::new("Are your disk partitioned?")
            .with_default(false)
            .prompt()
            .unwrap();

        if ans {
            break;
        }
    }

    // Select boot, root and swap partitions
    let output = Command::new("lsblk")
        .arg("-o")
        .arg("NAME,SIZE,MOUNTPOINTS")
        .output()
        .unwrap();

    let json = Command::new("lsblk").arg("-J").output().unwrap();
    let json: Value = serde_json::from_str(&String::from_utf8(json.stdout).unwrap()).unwrap();

    let mut options: Vec<&str> = Vec::new();
    for device in json["blockdevices"].as_array().unwrap() {
        if device["name"].as_str().unwrap() == partition {
            for subdevice in device["children"].as_array().unwrap() {
                options.push(subdevice["name"].as_str().unwrap());
            }
        }
    }

    let boot_partition = Select::new("Boot partition:", options.clone())
        .with_help_message(&String::from_utf8(output.stdout.clone()).unwrap())
        .prompt()
        .unwrap();

    let root_partition = Select::new("Root partition:", options.clone())
        .with_help_message(&String::from_utf8(output.stdout.clone()).unwrap())
        .prompt()
        .unwrap();

    let swap_partition = Select::new("Swap partition:", options)
        .with_help_message(&String::from_utf8(output.stdout.clone()).unwrap())
        .prompt()
        .unwrap();

    Command::new("mkfs.fat")
        .arg(format!("/dev/{boot_partition}"))
        .status()
        .unwrap();

    Command::new("mkfs.ext4")
        .arg(format!("/dev/{root_partition}"))
        .status()
        .unwrap();

    Command::new("mkswap")
        .arg(format!("/dev/{swap_partition}"))
        .status()
        .unwrap();

    Command::new("mount")
        .arg(format!("/dev/{root_partition}"))
        .arg("/mnt")
        .status()
        .unwrap();

    Command::new("mount")
        .arg("--mkdir")
        .arg(format!("/dev/{boot_partition}"))
        .arg("/mnt/boot")
        .status()
        .unwrap();

    Command::new("swapon")
        .arg(format!("/dev/{swap_partition}"))
        .status()
        .unwrap();

    let ans = Confirm::new("Are the partition mounted as expected?")
        .with_help_message(&String::from_utf8(output.stdout).unwrap())
        .with_default(true)
        .prompt()
        .unwrap();
    if !ans {
        panic!("Partitions not mounted as expected")
    }

    // Pacstrap
    let options = vec![
        "base",
        "base-devel",
        "linux",
        "linux-firmware",
        "networkmanager",
        "dhcpcd",
        "neovim",
        "nvidia",
    ];
    let ans = MultiSelect::new("New system packages:", options)
        .with_default(&[0, 1, 2, 3, 4, 5, 6])
        .prompt()
        .unwrap();

    Command::new("pacman")
        .args(["-Sy", "archlinux-keyring"])
        .status()
        .unwrap();

    Command::new("pacstrap")
        .arg("/mnt")
        .args(ans)
        .status()
        .unwrap();

    // Genfstab
    Command::new("genfstab")
        .arg("-U")
        .arg("/mnt")
        .arg(">>")
        .arg("/mnt/etc/fstab")
        .status()
        .unwrap();

    // Chroot
    Command::new("arch-chroot")
        .arg("/mnt")
        .args(["nvim", "/etc/locale.gen"])
        .status()
        .unwrap();

    let mut file = File::create("/mnt/etc/locale.conf").unwrap();
    file.write(b"LANG=en_US.UTF-8").unwrap();

    Command::new("locale-gen").status().unwrap();

    let hostname = Text::new("hostname:").prompt().unwrap();

    let mut file = File::create("/mnt/etc/hostname").unwrap();
    file.write(hostname.as_bytes()).unwrap();

    let hosts = format!(
        "
127.0.0.1   localhost
::1         localhost
127.0.1.1   {hostname}.localdomain  {hostname}"
    );

    let mut file = File::create("/mnt/etc/hosts").unwrap();
    file.write(hosts.as_bytes()).unwrap();

    Command::new("arch-chroot")
        .arg("/mnt")
        .arg("passwd")
        .status()
        .unwrap();

    let options = vec!["grub", "efibootmgr", "os-prober"];
    let ans = MultiSelect::new("boot config packages:", options)
        .with_default(&[0, 1, 2])
        .prompt()
        .unwrap();

    Command::new("arch-chroot")
        .arg("/mnt")
        .args(["pacman", "-S"])
        .args(ans)
        .status()
        .unwrap();

    Command::new("arch-chroot")
        .arg("/mnt")
        .args([
            "grub-install",
            "--target=x86_64-efi",
            "--efi-directory=/boot",
            "--bootloader-id=GRUB",
        ])
        .status()
        .unwrap();

    Command::new("arch-chroot")
        .arg("/mnt")
        .args(["nvim", "/etc/default/grub"])
        .status()
        .unwrap();

    Command::new("arch-chroot")
        .arg("/mnt")
        .args(["grub-mkconfig", "-o", "/boot/grub/grub.cfg"])
        .status()
        .unwrap();

    let username = Text::new("username").prompt().unwrap();
    Command::new("arch-chroot")
        .arg("/mnt")
        .args(["useradd", "-m", "-g", "wheel", &username])
        .status()
        .unwrap();

    Command::new("arch-chroot")
        .arg("/mnt")
        .args(["passwd", &username])
        .status()
        .unwrap();

    Command::new("arch-chroot")
        .arg("/mnt")
        .args(["nvim", "/etc/sudoers"])
        .status()
        .unwrap();

    println!("Minimarch installation complete!");
    println!("Here is an overview of what have been done:");
    println!("Coming soon...");
}

// ping google.com (check, if not ok, nmtui or smth)
// ls /sys/firmware/efi/efivars (check before anything)
// print lsblk (Select which drive to partition (get names with json format))
// cfdisk partition
// select boot partition (-> mkfs.fat)
// select root partition (-> mkfs.ext4)
// select swap partition (-> mkswap)
// mount /dev/root_partition /mnt
// mount --mkdir /dev/efi_system_partition /mnt/boot
// swapon /dev/swap_partition
// user check with lsblk
//
// multiselect (base base-devel linux linux-firmware networkmanager dhcpcd)
// select (query editor: nano, vim, neovim)
// pacstrap /mnt selections
//
// genfstab -U /mnt >> /mnt/etc/fstab
//
// arch/chroot /mnt
// nvim /etc/locale.gen
// check if it is ok (if not, reopen the file)
// locale-gen
// nvim /etc/locale.conf (write LANG=en_US.UTF-8)
//
// ask hostname (-> /etc/hostname)
// nvim /etc/hosts ->
// 127.0.0.1    localhost
// ::1          localhost
// 127.0.1.1    hostname.localdomain hostname
//
// passwd (change root password)
//
// pacman -S grub efibootmgr os-prober (confirm by user (y))
// grub-install --target=x86_64-efi --efi-directory=/boot --bootloader-id=GRUB
// nvim /etc/default/grub
// grub-mkconfig -o /boot/grub/grub.cfg
//
// ask username
// useradd -m -g wheel <user>
// passwd <user>
// nvim /etc/sudoers (uncomment %wheel ALL=(ALL:ALL) ALL)
//
// inquire nvidia, amd, etc...
