import os
os.environ['PYGAME_HIDE_SUPPORT_PROMPT'] = '1'
import pygame
import pygame._sdl2.audio as sdl2_audio
import subprocess
from typing import *



def get_devices(capture_devices: bool = False) -> Tuple[str, ...]:
    init_by_me = not pygame.mixer.get_init()
    if init_by_me:
        pygame.mixer.init()
    devices = tuple(sdl2_audio.get_audio_device_names(capture_devices))
    if init_by_me:
        pygame.mixer.quit()
    return devices


def play(file_path: str, device: Optional[str] = None, volume=1):
    if device is None:
        devices = get_devices()
        if not devices:
            raise RuntimeError("No device!")
        device = devices[0]
    print("Play: {}\r\nDevice: {}".format(file_path, device))
    pygame.mixer.init(devicename=device)
    pygame.mixer.music.load(file_path)
    pygame.mixer.music.set_volume(volume)
    pygame.mixer.music.play()
    #while pygame.mixer.music.get_busy():
    #    time.sleep(0.1)
    #pygame.mixer.quit()

def load_music(device_name, file_path, volume=1):
    pygame.mixer.init(devicename=device_name)
    pygame.mixer.music.load(file_path)
    pygame.mixer.music.set_volume(volume)

def play_music():#file_path: str), device: str, volume=1):
    pygame.mixer.music.play()

# message, sound, volume
clones = [
    #["#phone1", "resources/phone_ring.mp3", 1],
    #["#phone1", "resources/phone_ring.mp3", 1],
    #["#phone1", "resources/phone_ring.mp3", 1],
    #["#phone1", "resources/phone_ring.mp3", 1],
    #["#phone2", "resources/phone_ring.mp3", 1],
    #["#phone3", "resources/phone_ring.mp3", 1],
    ["#music1", "resources/music_box.mp3", 1],
    ["#music1", "resources/music_box.mp3", 1],
    ["#music1", "resources/music_box.mp3", 1],
    ["#music1", "resources/music_box.mp3", 1],
    ["#music1", "resources/music_box.mp3", 1],
    ["#music1", "resources/music_box.mp3", 1],
    ["#music1", "resources/music_box.mp3", 1],
]

nth_clone = 0
#else:
#    nth_clone = int(sys.argv[1])
#
#if nth_clone == len(clones):
#    sys.exit()
#
#subprocess.Popen(f"python3 {__file__} {nth_clone+1}", shell=True)
#
#device = nth_clone % len(get_devices())
#print(f"CLONE #{nth_clone}: {get_devices()[device]}")

msg, sound, volume = clones[nth_clone]
init_fast(get_devices()[device], sound, volume)

while True:
    _ = rx.receive(msg, block=True)
    play_fast()
    print("played sound fast")
    #play(sound, get_devices()[device], volume=volume)


