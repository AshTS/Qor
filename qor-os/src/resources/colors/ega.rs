use super::Pixel;

/* From Wikipedia (https://en.wikipedia.org/wiki/Enhanced_Graphics_Adapter)
    0	Black	#000000	000000	0
    1	Blue	#0000AA	000001	1
    2	Green	#00AA00	000010	2
    3	Cyan	#00AAAA	000011	3
    4	Red	#AA0000	000100	4
    5	Magenta	#AA00AA	000101	5
    6	Brown	#AA5500	010100	20
    7	White / light gray	#AAAAAA	000111	7
    8	Dark gray / bright black	#555555	111000	56
    9	Bright Blue	#5555FF	111001	57
    10	Bright green	#55FF55	111010	58
    11	Bright cyan	#55FFFF	111011	59
    12	Bright red	#FF5555	111100	60
    13	Bright magenta	#FF55FF	111101	61
    14	Bright yellow	#FFFF55	111110	62
    15	Bright white	#FFFFFF	111111	63 */

// EGA Colors
pub const EGA_COLORS: [Pixel; 16] = 
[
    Pixel::new(0, 0, 0),
    Pixel::new(0, 0, 0xAA),
    Pixel::new(0, 0xAA, 0),
    Pixel::new(0, 0xAA, 0xAA),
    Pixel::new(0xAA, 0, 0),
    Pixel::new(0xAA, 0, 0xAA),
    Pixel::new(0xAA, 0x55, 0),
    Pixel::new(0xAA, 0xAA, 0xAA),
    Pixel::new(0x55, 0x55, 0x55),
    Pixel::new(0x55, 0x55, 0xFF),
    Pixel::new(0x55, 0xFF, 0x55),
    Pixel::new(0x55, 0xFF, 0xFF),
    Pixel::new(0xFF, 0x55, 0x55),
    Pixel::new(0xFF, 0x55, 0xFF),
    Pixel::new(0xFF, 0xFF, 0x55),
    Pixel::new(0xFF, 0xFF, 0xFF),
];  