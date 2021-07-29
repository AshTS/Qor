import PIL

image = PIL.Image.open("image.png")
px = image.load()

start = 3, 3

def get_col(px, x, y, col):
  X = x * 9 + start[0]
  Y = y * 16 + start[1]

  result = 0

  for yv in range(16):
    result <<= 1
    if px[X + col, Y + yv] > 0:
      result |= 1

  return result

def render_symbol(px, x, y):
  print("        [ // 0x{:02X}".format(x + y * 32))
  for i in range(9):
    print("            0b{:016b},".format(get_col(px, x, y, i)))
  print("        ],")
  
print("pub const GLYPHS: [[u16; 9]; 256] = ")
print("    [")

for y in range(8):
  for x in range(32):
    render_symbol(px, x, y)

print("    ];")