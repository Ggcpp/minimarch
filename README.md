# Guide

## Wifi

1. ```iwctl```
2. ```device list```
3. ```station <device_name> scan```
4. ```station <device_name> get-networks```
5. ```station <device_name> connect <network_name>```
6. ```ping google.com``` (to check if the wifi works)

## Installing minimarch

1. ```curl -L -O https://github.com/Ggcpp/minimarch/releases/download/Test/minimarch```
2. ```chmod +x minimarch```

## Installing Arch

1. ```./minimarch```
2. Follow the instructions (**you should make only 3 partitions: [Boot, Swap, Root]. There is no separate Home partition**)
3. When the file ```/etc/locale.gen``` opens up, uncomment ```en_US.UTF-8 UTF-8```
4. ```hostname``` will be the name of the machine
5. When the file ```/etc/default/grub``` opens up, set the ```GRUB_TIMEOUT``` to whatever (```-1``` means no timeout) and uncomment ```GRUB_DISABLE_OS_PROBER``` is you are dual booting
6. When the file ```/etc/sudoers``` opens up, uncomment ```%wheel ALL=(ALL:ALL) ALL```
7. If the installation in complete, you can ```reboot```

## Bonus

After reboot, you can set the timezone with ```timedatectl set-timezone <Tab to see timezones>```
