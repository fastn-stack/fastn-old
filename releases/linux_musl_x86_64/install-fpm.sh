# usr/bin/sh

mkdir -p /tmp/fpm/bin/
curl https://github.com/FifthTry/fpm/raw/releases/releases/linux_musl_x86_64/fpm -L -o /tmp/fpm/bin/fpm
curl https://github.com/FifthTry/fpm/raw/releases/releases/linux_musl_x86_64/fpm.d -L -o /tmp/fpm/bin/fpm.d
chmod +x /tmp/fpm/bin/fpm*
