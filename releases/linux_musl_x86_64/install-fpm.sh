# usr/bin/sh

curl -s https://api.github.com/repos/fifthtry/fpm/releases/latest | grep ".*\/releases\/download\/.*\ fpm_linux[A-z_]*" | cut -d : -f 2,3 | xargs -I % curl -L % -o /usr/bin/fpm
curl -s https://api.github.com/repos/fifthtry/fpm/releases/latest | grep ".*\/releases\/download\/.*\/fpm_linux.*.d" | cut -d : -f 2,3 | xargs -I % curl -L % -o /usr/bin/fpm.d
chmod +x /usr/bin/fpm*
