import os
os.environ['PYGAME_HIDE_SUPPORT_PROMPT'] = '1'
import pygame
import pygame._sdl2.audio as sdl2_audio
import subprocess
from typing import *
from subprocess import Popen
import time

import dave
import sys

def get_devices() -> Tuple[str, ...]:
    init_by_me = not pygame.mixer.get_init()
    if init_by_me:
        pygame.mixer.init()
    devices = tuple(sdl2_audio.get_audio_device_names(False))
    if init_by_me:
        pygame.mixer.quit()
        #^this is necessary for some reason
    return devices

def load_music(device_name: str, file_path: str, volume=1):
    pygame.mixer.music.stop()
    pygame.mixer.quit()

    pygame.mixer.init(devicename=device_name)

    pygame.mixer.music.load(file_path)
    pygame.mixer.music.set_volume(volume)

def play_music():
    pygame.mixer.music.play()

# message, sound, volume
clones = [
    ["#music1", "resources/music_box.mp3", 1],
]

# if no arguments passed, this is the root script.
# for every device found,
#   spawn three scripts each listening for a different msg:
#       %track1
#       %track2
#       %track3

#nth_clone = 0
#
#device = nth_clone % len(get_devices())
#
#msg, sound, volume = clones[nth_clone]
#load_music(get_devices()[device], sound, volume)

def repl():
    procs = []
    devices = []
    for d in get_devices():
        if "Meta" in d or "Teams" in d:
            continue
        devices.append(d)

    tracks = ["%track1", "%track2", "%track3"]
    for d in devices:
        for t in tracks:
            procs.append(Popen([f"python3 {__file__} '{d}' '{t}'"], shell=True))

    mt = dave.MemberTable()
    _ = [mt.preload(t) for t in tracks]

    time.sleep(1)
    while True:
        cmd = input("> ").strip().split(' ')
        if cmd == "STOP":
            mt.send(track, data=f"{resource} {volume}")
            continue
        if cmd == "TERMINATE":
            [p.terminate() for p in procs]
            break

        if len(cmd) < 3:
            print("not enough args (need 3)")
            continue
        track = cmd[0]
        if int(track) > 3 or int(track) < 1:
            print("invalid track: {track}")
            continue
        track = f"%track{track}"
        resource = f"resources/{cmd[1]}.mp3"
        if not os.path.exists(resource):
            print(f"resource does not exist: {resource}")

        volume = cmd[2]

        print(f"sending: {track} {resource}")
        mt.send(track, data=f"{resource} {volume}")
        time.sleep(0.5)

def child(device, track):
    print(f"SCRIPT SPAWNED: {device} w/ {track}")
    pygame.mixer.init()
    mt = dave.MemberTable()

    while True:
        sound_path = mt.recv(track)
        print(f"received path: {sound_path}")
        if sound_path == "STOP":
            break
        sound_path, volume = sound_path.split(' ')

        load_music(device, sound_path, float(volume))
        play_music()

    

if len(sys.argv) == 1:
    repl() #root script
else:
    child(sys.argv[1], sys.argv[2]) #child script



#while True:
#    play_music()
#    print("played sound fast")
#    #play(sound, get_devices()[device], volume=volume)


