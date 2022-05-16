import pygame
from hilbertcurve.hilbertcurve import HilbertCurve
import os
pygame.init()

dim = 2
# Every higher-level image has 256 lower images -- Level 8 Hilbert curve
level = 8
curve = HilbertCurve(level, dim)

for a in range(256):
    print(a)
    #surf = pygame.Surface((4096,4096))  # 2**24 pixels
    #surf.fill((64,0,0))
    surf = None
    did_change = False
    for b in range(256):
        chunk_x, chunk_y = curve.point_from_distance(b)
        px_x = chunk_x * 256
        px_y = chunk_y * 256
        try:
            img = pygame.image.load(f'images/{a}/{b}.png')
            if surf is None:
                surf = pygame.Surface((4096,4096))
                surf.fill((64,0,0))
            surf.blit(img, (px_x, px_y))
            did_change = True
        except FileNotFoundError:
            pass
    if did_change:
        pygame.image.save(surf, f'images/{a}.png')