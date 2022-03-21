#! /bin/python3

import json
import os
import subprocess
import sys

cwd = ""

binary_setup_path = "qor-userland/build.json"

"""
binaries = [{"name": "LibC", "make-path": "libc", "bin-path": "libc/bin/libc.a", "output-path": "/lib/libc.a", "postbuild": ["cp libc/bin/libc.a qor-userland/lib/libc.a"]},
            ]#{"name": "Hello", "make-path": "qor-userland/examples/hello", "bin-path": "qor-userland/examples/hello/bin/hello", "output-path": "/bin/hello"}]
"""

binaries = json.load(open(binary_setup_path, "r"))

def decons(a):
    return a[0], a[1:]

def usage(prog_name):
    print("USAGE: %s <subcommand> [ARGS]" % prog_name)
    print("")
    print("   Subcommands:")
    print("     clean                Delete all binaries and libraries")
    print("     build                Build all of the userland programs")
    print("     build-run            Build all of the userland programs and then run")
    print("     disk                 Move necessary files to disk")
    print("     rebuild              Delete all binaries and libraries, and then build")
    print("     run                  Run Qor")
    print("     update               Update header files")

def expect_arg(args, to_expect, prog_name):
    if len(args) == 0:
        usage(prog_name)
        print("ERROR: Expected", to_expect)
        sys.exit(1)

    return decons(args)

def run_command(command, show=True, hide_path=True, no_capture=False, hide=False, **kwargs):
    if not "cwd" in kwargs:
        kwargs["cwd"] = cwd
    
    if show:
        if not hide_path:
            d = "." + kwargs["cwd"].split(cwd)[1]
            print("  ", d)
        print("    ", command)
    
    if not no_capture:
        kwargs["text"] = True
        kwargs["capture_output"] = True

    result = subprocess.run(command, **kwargs)

    if not no_capture:
        if not hide:
            for line in result.stdout.split("\n"):
                if line == "":
                    continue
                print("      ", line)

            for line in result.stderr.split("\n"):
                if line == "":
                    continue
                print("      ", line)

    return result

def build_path(path, name):
    if run_command("make -q", show=False, shell=True, cwd=cwd+path).returncode > 0:
        print("Building", name)
        result = run_command("make", shell=True, hide_path=True, cwd=cwd+path, text=True, no_capture=True)

        result.check_returncode()

        return True

    return False

def clean_path(path, name):
    print("Removing", name)
    run_command("make clean", shell=True, cwd=cwd+path).check_returncode()

def clean():
    print("Removing Binaries")

    for entry in binaries:
        clean_path("/" + entry["make-path"], entry["name"])

def build():
    print("Ensuring Directories Exist")
    run_command("mkdir qor-userland/lib", shell=True, hide=True)

    print("Building Binaries")

    for entry in binaries:
        if build_path("/" + entry["make-path"], entry["name"]):
            if "postbuild" in entry:
                cmds = entry["postbuild"]

                print("Running post build for", entry["name"])

                for cmd in cmds:
                    run_command(cmd, shell=True).check_returncode()

def update_headers():
    print("Updating Headers")

    run_command("mkdir qor-userland/include/libc", shell=True, hide=True)
    run_command("cp libc/include/* qor-userland/include/libc/ -r", shell=True, cwd=cwd).check_returncode()

def mount_disk():
    run_command("sudo losetup /dev/loop16 qor-os/hdd.dsk; sudo mount /dev/loop16 /mnt", show=True, shell=True).check_returncode()

def unmount_disk():
    run_command("sudo sync /mnt; sudo umount /mnt; sudo losetup -d /dev/loop16", show=True, shell=True).check_returncode()

def update_disk():
    print("Copying files to Disk")

    mount_disk()

    try:
        run_command("sudo rm -rf /mnt/*", shell=True, hide=True)
        run_command("sudo mkdir /mnt/lib", shell=True, hide=True)
        run_command("sudo mkdir /mnt/bin", shell=True, hide=True)

        run_command("sudo cp -rp qor-userland/root/* /mnt", shell=True, hide=True).check_returncode()

        for entry in binaries:
            run_command("sudo cp -p " + entry["bin-path"] + " /mnt" + entry["output-path"], shell=True).check_returncode()
    finally:
        unmount_disk()

def run():
    print("Starting Qor")

    run_command("cargo run --release", cwd=cwd+"/qor-os", shell=True, no_capture=True).check_returncode()

    copy_output()

def copy_output():
    print("Copying output")

    mount_disk()

    try:
        run_command("rm -rf qor-userland/root-output", shell=True, hide=True)
        run_command("mkdir qor-userland/root-output", shell=True, hide=True)
        run_command("cp -rp /mnt/home/root qor-userland/root-output", shell=True, hide=True).check_returncode()
    finally:
        unmount_disk()

if __name__ == "__main__":
    prog_name, args = decons(sys.argv)

    subcommand, args = expect_arg(args, "subcommand", prog_name)

    cwd = subprocess.run(["pwd"], shell=True, text=True, capture_output=True).stdout.strip()

    os.environ['qorIncludePath'] = cwd + "/qor-userland/include"
    os.environ['qorLibPath'] = cwd + "/qor-userland/lib"

    if subcommand == "clean":
        clean()
    elif subcommand == "rebuild":
        clean()
        update_headers()
        build()
        update_disk()
    elif subcommand == "build":
        build()
        update_disk()
    elif subcommand == "build-run":
        build()
        update_disk()
        run()
    elif subcommand == "update":
        update_headers()
    elif subcommand == "disk":
        update_disk()
    elif subcommand == "run":
        run()
    else:
        usage(prog_name)
        print("ERROR: Unknown subcommand", subcommand)
        sys.exit(1)

