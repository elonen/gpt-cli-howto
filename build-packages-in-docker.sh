#!/bin/bash
set -e
VERSION=$(cargo metadata --no-deps --format-version 1 | jq -r '.packages[0] | [ .version ] | join("-")')

IMG="gpt-cli-howto-build"

if [[ ! " $@ " =~ " --skip-docker-build " ]]; then
    echo "=========== Build docker image ==========="
    docker build -t $IMG .
fi

#echo "=========== Make .exe ==========="
#docker run --rm -iv${PWD}:/root/OUTPUT $IMG bash -xvs << EOF
#    set -e
#    cd /root
#    PYO3_CROSS_PYTHON_VERSION=3.9 cargo build --target x86_64-pc-windows-gnu --release --verbose || exit 1
#    chown -v $(id -u):$(id -g)  target/x86_64-pc-windows-gnu/release/*.exe
#    cp -va target/x86_64-pc-windows-gnu/release/howto.exe OUTPUT/howto.exe
#    echo "============ Done. Built for: ============="
#    lsb_release -a
#EOF

echo "=========== Make .deb ==========="
docker run --rm -iv${PWD}:/root/OUTPUT $IMG bash -xvs << EOF
    set -e
    cd /root
    cargo deb --verbose || exit 1
    chown -v $(id -u):$(id -g) target/debian/*.deb
    cp -va target/debian/*.deb OUTPUT/
    cp -va target/release/howto OUTPUT/
    echo "============ Done. Built for: ============="
    lsb_release -a
    echo "\n...and x86_64-pc-windows-gnu"
EOF

echo "=============== $(pwd) ==============="
#ls -l *.deb *.exe
ls -l *.deb
