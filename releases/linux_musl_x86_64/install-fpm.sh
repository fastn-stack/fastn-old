# usr/bin/sh

mkdir -p /tmp/fpm/bin/
curl -s https://api.github.com/repos/fifthtry/fpm/releases/latest | grep ".*\/releases\/download\/.*\ fpm_linux[A-z_]*" | cut -d : -f 2,3 | xargs -I % curl -L % -o /tmp/fpm/bin/fpm
curl -s https://api.github.com/repos/fifthtry/fpm/releases/latest | grep ".*\/releases\/download\/.*\/fpm_linux.*.d" | cut -d : -f 2,3 | xargs -I % curl -L % -o /tmp/fpm/bin/fpm.d
chmod +x /tmp/fpm/bin/fpm*
