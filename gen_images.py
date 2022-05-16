import pygame
from hilbertcurve.hilbertcurve import HilbertCurve
import os
pygame.init()

dim = 2
# Every file in Level2HostStateDatabase has x.x.0.0 -- x.x.255.255 (65536) -- Level 16 Hilbert curve
level = 16
curve = HilbertCurve(level, dim)

def iter_bits(file):
    byte = file.read(1)
    while byte != b'':
        for i in range(8):
            yield byte[0] >> i & 1
        byte = file.read(1)

for a in range(256):
    for b in range(256):
        try:
            # If the image is newer than the binary, skip rerendering it
            if os.stat(f'images/{a}/{b}.png').st_mtime > os.stat(f'data/{a}/{b}.bin').st_mtime:
                continue
        except FileNotFoundError:
            pass
        
        try:
            with open(f'data/{a}/{b}.bin', 'rb') as o:
                print(a, b)

                surf = pygame.Surface((256,256))  # 65536 pixels
                surf.fill((0,0,0))
                for index, bit in enumerate(iter_bits(o)):
                    if bit:
                        x, y = curve.point_from_distance(index)
                        surf.set_at((x,y), (255,255,255))
                pygame.image.save(surf, f'images/{a}/{b}.png')
        except FileNotFoundError:
            pass