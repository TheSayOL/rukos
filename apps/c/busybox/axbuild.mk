app-objs=main.o

ARGS = /bin/busybox,sh
ENVS = 
V9P_PATH=${APP}/rootfs
# make run ARCH=aarch64 A=apps/c/busybox V9P=y MUSL=y LOG=debug