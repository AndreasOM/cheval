# napkin math

ARGB ARGB ARGB ARGB

0RGB 0RGB 0RGB 0RGB

0AAA 0AAA 0AAA 0AAA


AAAA RRRR GGGG BBBB

 0xff 0x000000ff
+0xff 0x000000ff
+0xff 0x000000ff
+0xff 0x000000ff
 0xfc 0x000003fc
/4
 0x3c 0x000000ff

// without "super sampling"

 ARGB

 0RGB
 AAAA


 // with "super sampling"

 0xff 0x00ff
+0xff 0x00ff
+0xff 0x00ff
+0xff 0x00ff
 0xfc 0x03fc
/4
 0x3c 0x00ff

ARGB ARGB ARGB ARGB
+ 000B 000B 000B 000B
= 00BB
/4
= 00BB

    ARGB ARGB ARGB ARGB
&   0F0F
=   0R0B 0R0B 0R0B 0R0B
+   .... .... .... ....
=   RRBB
>>2 0R0B

&   F0F0
=   A0G0 ...........
>>8 0A0G .....
+   ..........
=   AAGG AAGG AAGG AAGG
>>2 0A0G

Before:
16 >>
16 &
16 +
1  /4 or >>2
3 &
3 <<
2 |
3 &
3 <<
2 |

23 << + >>
22 &
16 +
4 |
65

After:
4 &
3 +
4 >>
4 &
4 >>
3 +
3 <<
2 |

11 << + >>
8 &
6 +
2 |
27






