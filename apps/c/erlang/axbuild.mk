app-objs=main.o

NET=y

ARGS = /bin/erlexec
ENVS = EMU=beam,BINDIR=/bin
#ROOTDIR=/,PROGNAME=erl
V9P_PATH=${APP}/rootfs

# make run ARCH=aarch64 A=apps/c/erlang V9P=y MUSL=y LOG=debug