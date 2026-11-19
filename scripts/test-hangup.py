#!/usr/bin/env python3
"""Prueba de extremo a extremo del arreglo 0.5.4: si la terminal desaparece,
rsi debe terminar solo (antes quedaba girando al 100 % de un núcleo).

Uso: scripts/test-hangup.py [ruta-al-binario]   (por defecto target/release/rsi)
"""
import fcntl
import os
import pty
import signal
import struct
import sys
import termios
import time

BIN = sys.argv[1] if len(sys.argv) > 1 else "target/release/rsi"
ARGS = sys.argv[2:]
START = 1.5  # segundos hasta que rsi está dibujando
LIMIT = 8.0  # plazo máximo para salir tras colgar la terminal


def alive(pid):
    done, _ = os.waitpid(pid, os.WNOHANG)
    return done == 0


def main():
    pid, fd = pty.fork()
    if pid == 0:
        os.environ["TERM"] = "xterm-256color"
        os.execv(BIN, [BIN, *ARGS])
    fcntl.ioctl(fd, termios.TIOCSWINSZ, struct.pack("HHHH", 40, 120, 0, 0))
    time.sleep(START)
    if not alive(pid):
        print("FALLO: el proceso terminó antes de colgar la terminal")
        return 1
    os.close(fd)  # la terminal "desaparece"
    start = time.time()
    while time.time() - start < LIMIT:
        done, _ = os.waitpid(pid, os.WNOHANG)
        if done:
            print(f"ok: terminó {time.time() - start:.1f} s después del cuelgue")
            return 0
        time.sleep(0.1)
    os.kill(pid, signal.SIGKILL)
    os.waitpid(pid, 0)
    print(f"FALLO: sigue vivo {LIMIT:.0f} s después de colgar la terminal")
    return 1


sys.exit(main())
