// +-----------------------------------------------------------------------------------------------+
// | Copyright 2016 Sean Kerr                                                                      |
// |                                                                                               |
// | Licensed under the Apache License, Version 2.0 (the "License);                               |
// | you may not use this file except in compliance with the License.                              |
// | You may obtain a copy of the License at                                                       |
// |                                                                                               |
// |  http://www.apache.org/licenses/LICENSE-2.0                                                   |
// |                                                                                               |
// | Unless required by applicable law or agreed to in writing, software                           |
// | distributed under the License is distributed on an "AS IS" BASIS,                             |
// | WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.                      |
// | See the License for the specific language governing permissions and                           |
// | limitations under the License.                                                                |
// +-----------------------------------------------------------------------------------------------+
// | Author: Sean Kerr <sean@code-box.org>                                                         |
// +-----------------------------------------------------------------------------------------------+

use byte::*;

#[test]
#[allow(cyclomatic_complexity)]
fn hex_to_byte_valid() {
    assert_eq!(hex_to_byte(b"0").unwrap(), 0x00);
    assert_eq!(hex_to_byte(b"1").unwrap(), 0x01);
    assert_eq!(hex_to_byte(b"2").unwrap(), 0x02);
    assert_eq!(hex_to_byte(b"3").unwrap(), 0x03);
    assert_eq!(hex_to_byte(b"4").unwrap(), 0x04);
    assert_eq!(hex_to_byte(b"5").unwrap(), 0x05);
    assert_eq!(hex_to_byte(b"6").unwrap(), 0x06);
    assert_eq!(hex_to_byte(b"7").unwrap(), 0x07);
    assert_eq!(hex_to_byte(b"8").unwrap(), 0x08);
    assert_eq!(hex_to_byte(b"9").unwrap(), 0x09);
    assert_eq!(hex_to_byte(b"A").unwrap(), 0x0A);
    assert_eq!(hex_to_byte(b"B").unwrap(), 0x0B);
    assert_eq!(hex_to_byte(b"C").unwrap(), 0x0C);
    assert_eq!(hex_to_byte(b"D").unwrap(), 0x0D);
    assert_eq!(hex_to_byte(b"E").unwrap(), 0x0E);
    assert_eq!(hex_to_byte(b"F").unwrap(), 0x0F);
    assert_eq!(hex_to_byte(b"00").unwrap(), 0x00);
    assert_eq!(hex_to_byte(b"01").unwrap(), 0x01);
    assert_eq!(hex_to_byte(b"02").unwrap(), 0x02);
    assert_eq!(hex_to_byte(b"03").unwrap(), 0x03);
    assert_eq!(hex_to_byte(b"04").unwrap(), 0x04);
    assert_eq!(hex_to_byte(b"05").unwrap(), 0x05);
    assert_eq!(hex_to_byte(b"06").unwrap(), 0x06);
    assert_eq!(hex_to_byte(b"07").unwrap(), 0x07);
    assert_eq!(hex_to_byte(b"08").unwrap(), 0x08);
    assert_eq!(hex_to_byte(b"09").unwrap(), 0x09);
    assert_eq!(hex_to_byte(b"0A").unwrap(), 0x0A);
    assert_eq!(hex_to_byte(b"0B").unwrap(), 0x0B);
    assert_eq!(hex_to_byte(b"0C").unwrap(), 0x0C);
    assert_eq!(hex_to_byte(b"0D").unwrap(), 0x0D);
    assert_eq!(hex_to_byte(b"0E").unwrap(), 0x0E);
    assert_eq!(hex_to_byte(b"0F").unwrap(), 0x0F);
    assert_eq!(hex_to_byte(b"10").unwrap(), 0x10);
    assert_eq!(hex_to_byte(b"11").unwrap(), 0x11);
    assert_eq!(hex_to_byte(b"12").unwrap(), 0x12);
    assert_eq!(hex_to_byte(b"13").unwrap(), 0x13);
    assert_eq!(hex_to_byte(b"14").unwrap(), 0x14);
    assert_eq!(hex_to_byte(b"15").unwrap(), 0x15);
    assert_eq!(hex_to_byte(b"16").unwrap(), 0x16);
    assert_eq!(hex_to_byte(b"17").unwrap(), 0x17);
    assert_eq!(hex_to_byte(b"18").unwrap(), 0x18);
    assert_eq!(hex_to_byte(b"19").unwrap(), 0x19);
    assert_eq!(hex_to_byte(b"1A").unwrap(), 0x1A);
    assert_eq!(hex_to_byte(b"1B").unwrap(), 0x1B);
    assert_eq!(hex_to_byte(b"1C").unwrap(), 0x1C);
    assert_eq!(hex_to_byte(b"1D").unwrap(), 0x1D);
    assert_eq!(hex_to_byte(b"1E").unwrap(), 0x1E);
    assert_eq!(hex_to_byte(b"1F").unwrap(), 0x1F);
    assert_eq!(hex_to_byte(b"20").unwrap(), 0x20);
    assert_eq!(hex_to_byte(b"21").unwrap(), 0x21);
    assert_eq!(hex_to_byte(b"22").unwrap(), 0x22);
    assert_eq!(hex_to_byte(b"23").unwrap(), 0x23);
    assert_eq!(hex_to_byte(b"24").unwrap(), 0x24);
    assert_eq!(hex_to_byte(b"25").unwrap(), 0x25);
    assert_eq!(hex_to_byte(b"26").unwrap(), 0x26);
    assert_eq!(hex_to_byte(b"27").unwrap(), 0x27);
    assert_eq!(hex_to_byte(b"28").unwrap(), 0x28);
    assert_eq!(hex_to_byte(b"29").unwrap(), 0x29);
    assert_eq!(hex_to_byte(b"2A").unwrap(), 0x2A);
    assert_eq!(hex_to_byte(b"2B").unwrap(), 0x2B);
    assert_eq!(hex_to_byte(b"2C").unwrap(), 0x2C);
    assert_eq!(hex_to_byte(b"2D").unwrap(), 0x2D);
    assert_eq!(hex_to_byte(b"2E").unwrap(), 0x2E);
    assert_eq!(hex_to_byte(b"2F").unwrap(), 0x2F);
    assert_eq!(hex_to_byte(b"30").unwrap(), 0x30);
    assert_eq!(hex_to_byte(b"31").unwrap(), 0x31);
    assert_eq!(hex_to_byte(b"32").unwrap(), 0x32);
    assert_eq!(hex_to_byte(b"33").unwrap(), 0x33);
    assert_eq!(hex_to_byte(b"34").unwrap(), 0x34);
    assert_eq!(hex_to_byte(b"35").unwrap(), 0x35);
    assert_eq!(hex_to_byte(b"36").unwrap(), 0x36);
    assert_eq!(hex_to_byte(b"37").unwrap(), 0x37);
    assert_eq!(hex_to_byte(b"38").unwrap(), 0x38);
    assert_eq!(hex_to_byte(b"39").unwrap(), 0x39);
    assert_eq!(hex_to_byte(b"3A").unwrap(), 0x3A);
    assert_eq!(hex_to_byte(b"3B").unwrap(), 0x3B);
    assert_eq!(hex_to_byte(b"3C").unwrap(), 0x3C);
    assert_eq!(hex_to_byte(b"3D").unwrap(), 0x3D);
    assert_eq!(hex_to_byte(b"3E").unwrap(), 0x3E);
    assert_eq!(hex_to_byte(b"3F").unwrap(), 0x3F);
    assert_eq!(hex_to_byte(b"40").unwrap(), 0x40);
    assert_eq!(hex_to_byte(b"41").unwrap(), 0x41);
    assert_eq!(hex_to_byte(b"42").unwrap(), 0x42);
    assert_eq!(hex_to_byte(b"43").unwrap(), 0x43);
    assert_eq!(hex_to_byte(b"44").unwrap(), 0x44);
    assert_eq!(hex_to_byte(b"45").unwrap(), 0x45);
    assert_eq!(hex_to_byte(b"46").unwrap(), 0x46);
    assert_eq!(hex_to_byte(b"47").unwrap(), 0x47);
    assert_eq!(hex_to_byte(b"48").unwrap(), 0x48);
    assert_eq!(hex_to_byte(b"49").unwrap(), 0x49);
    assert_eq!(hex_to_byte(b"4A").unwrap(), 0x4A);
    assert_eq!(hex_to_byte(b"4B").unwrap(), 0x4B);
    assert_eq!(hex_to_byte(b"4C").unwrap(), 0x4C);
    assert_eq!(hex_to_byte(b"4D").unwrap(), 0x4D);
    assert_eq!(hex_to_byte(b"4E").unwrap(), 0x4E);
    assert_eq!(hex_to_byte(b"4F").unwrap(), 0x4F);
    assert_eq!(hex_to_byte(b"50").unwrap(), 0x50);
    assert_eq!(hex_to_byte(b"51").unwrap(), 0x51);
    assert_eq!(hex_to_byte(b"52").unwrap(), 0x52);
    assert_eq!(hex_to_byte(b"53").unwrap(), 0x53);
    assert_eq!(hex_to_byte(b"54").unwrap(), 0x54);
    assert_eq!(hex_to_byte(b"55").unwrap(), 0x55);
    assert_eq!(hex_to_byte(b"56").unwrap(), 0x56);
    assert_eq!(hex_to_byte(b"57").unwrap(), 0x57);
    assert_eq!(hex_to_byte(b"58").unwrap(), 0x58);
    assert_eq!(hex_to_byte(b"59").unwrap(), 0x59);
    assert_eq!(hex_to_byte(b"5A").unwrap(), 0x5A);
    assert_eq!(hex_to_byte(b"5B").unwrap(), 0x5B);
    assert_eq!(hex_to_byte(b"5C").unwrap(), 0x5C);
    assert_eq!(hex_to_byte(b"5D").unwrap(), 0x5D);
    assert_eq!(hex_to_byte(b"5E").unwrap(), 0x5E);
    assert_eq!(hex_to_byte(b"5F").unwrap(), 0x5F);
    assert_eq!(hex_to_byte(b"60").unwrap(), 0x60);
    assert_eq!(hex_to_byte(b"61").unwrap(), 0x61);
    assert_eq!(hex_to_byte(b"62").unwrap(), 0x62);
    assert_eq!(hex_to_byte(b"63").unwrap(), 0x63);
    assert_eq!(hex_to_byte(b"64").unwrap(), 0x64);
    assert_eq!(hex_to_byte(b"65").unwrap(), 0x65);
    assert_eq!(hex_to_byte(b"66").unwrap(), 0x66);
    assert_eq!(hex_to_byte(b"67").unwrap(), 0x67);
    assert_eq!(hex_to_byte(b"68").unwrap(), 0x68);
    assert_eq!(hex_to_byte(b"69").unwrap(), 0x69);
    assert_eq!(hex_to_byte(b"6A").unwrap(), 0x6A);
    assert_eq!(hex_to_byte(b"6B").unwrap(), 0x6B);
    assert_eq!(hex_to_byte(b"6C").unwrap(), 0x6C);
    assert_eq!(hex_to_byte(b"6D").unwrap(), 0x6D);
    assert_eq!(hex_to_byte(b"6E").unwrap(), 0x6E);
    assert_eq!(hex_to_byte(b"6F").unwrap(), 0x6F);
    assert_eq!(hex_to_byte(b"70").unwrap(), 0x70);
    assert_eq!(hex_to_byte(b"71").unwrap(), 0x71);
    assert_eq!(hex_to_byte(b"72").unwrap(), 0x72);
    assert_eq!(hex_to_byte(b"73").unwrap(), 0x73);
    assert_eq!(hex_to_byte(b"74").unwrap(), 0x74);
    assert_eq!(hex_to_byte(b"75").unwrap(), 0x75);
    assert_eq!(hex_to_byte(b"76").unwrap(), 0x76);
    assert_eq!(hex_to_byte(b"77").unwrap(), 0x77);
    assert_eq!(hex_to_byte(b"78").unwrap(), 0x78);
    assert_eq!(hex_to_byte(b"79").unwrap(), 0x79);
    assert_eq!(hex_to_byte(b"7A").unwrap(), 0x7A);
    assert_eq!(hex_to_byte(b"7B").unwrap(), 0x7B);
    assert_eq!(hex_to_byte(b"7C").unwrap(), 0x7C);
    assert_eq!(hex_to_byte(b"7D").unwrap(), 0x7D);
    assert_eq!(hex_to_byte(b"7E").unwrap(), 0x7E);
    assert_eq!(hex_to_byte(b"7F").unwrap(), 0x7F);
    assert_eq!(hex_to_byte(b"80").unwrap(), 0x80);
    assert_eq!(hex_to_byte(b"81").unwrap(), 0x81);
    assert_eq!(hex_to_byte(b"82").unwrap(), 0x82);
    assert_eq!(hex_to_byte(b"83").unwrap(), 0x83);
    assert_eq!(hex_to_byte(b"84").unwrap(), 0x84);
    assert_eq!(hex_to_byte(b"85").unwrap(), 0x85);
    assert_eq!(hex_to_byte(b"86").unwrap(), 0x86);
    assert_eq!(hex_to_byte(b"87").unwrap(), 0x87);
    assert_eq!(hex_to_byte(b"88").unwrap(), 0x88);
    assert_eq!(hex_to_byte(b"89").unwrap(), 0x89);
    assert_eq!(hex_to_byte(b"8A").unwrap(), 0x8A);
    assert_eq!(hex_to_byte(b"8B").unwrap(), 0x8B);
    assert_eq!(hex_to_byte(b"8C").unwrap(), 0x8C);
    assert_eq!(hex_to_byte(b"8D").unwrap(), 0x8D);
    assert_eq!(hex_to_byte(b"8E").unwrap(), 0x8E);
    assert_eq!(hex_to_byte(b"8F").unwrap(), 0x8F);
    assert_eq!(hex_to_byte(b"90").unwrap(), 0x90);
    assert_eq!(hex_to_byte(b"91").unwrap(), 0x91);
    assert_eq!(hex_to_byte(b"92").unwrap(), 0x92);
    assert_eq!(hex_to_byte(b"93").unwrap(), 0x93);
    assert_eq!(hex_to_byte(b"94").unwrap(), 0x94);
    assert_eq!(hex_to_byte(b"95").unwrap(), 0x95);
    assert_eq!(hex_to_byte(b"96").unwrap(), 0x96);
    assert_eq!(hex_to_byte(b"97").unwrap(), 0x97);
    assert_eq!(hex_to_byte(b"98").unwrap(), 0x98);
    assert_eq!(hex_to_byte(b"99").unwrap(), 0x99);
    assert_eq!(hex_to_byte(b"9A").unwrap(), 0x9A);
    assert_eq!(hex_to_byte(b"9B").unwrap(), 0x9B);
    assert_eq!(hex_to_byte(b"9C").unwrap(), 0x9C);
    assert_eq!(hex_to_byte(b"9D").unwrap(), 0x9D);
    assert_eq!(hex_to_byte(b"9E").unwrap(), 0x9E);
    assert_eq!(hex_to_byte(b"9F").unwrap(), 0x9F);
    assert_eq!(hex_to_byte(b"A0").unwrap(), 0xA0);
    assert_eq!(hex_to_byte(b"A1").unwrap(), 0xA1);
    assert_eq!(hex_to_byte(b"A2").unwrap(), 0xA2);
    assert_eq!(hex_to_byte(b"A3").unwrap(), 0xA3);
    assert_eq!(hex_to_byte(b"A4").unwrap(), 0xA4);
    assert_eq!(hex_to_byte(b"A5").unwrap(), 0xA5);
    assert_eq!(hex_to_byte(b"A6").unwrap(), 0xA6);
    assert_eq!(hex_to_byte(b"A7").unwrap(), 0xA7);
    assert_eq!(hex_to_byte(b"A8").unwrap(), 0xA8);
    assert_eq!(hex_to_byte(b"A9").unwrap(), 0xA9);
    assert_eq!(hex_to_byte(b"AA").unwrap(), 0xAA);
    assert_eq!(hex_to_byte(b"AB").unwrap(), 0xAB);
    assert_eq!(hex_to_byte(b"AC").unwrap(), 0xAC);
    assert_eq!(hex_to_byte(b"AD").unwrap(), 0xAD);
    assert_eq!(hex_to_byte(b"AE").unwrap(), 0xAE);
    assert_eq!(hex_to_byte(b"AF").unwrap(), 0xAF);
    assert_eq!(hex_to_byte(b"B0").unwrap(), 0xB0);
    assert_eq!(hex_to_byte(b"B1").unwrap(), 0xB1);
    assert_eq!(hex_to_byte(b"B2").unwrap(), 0xB2);
    assert_eq!(hex_to_byte(b"B3").unwrap(), 0xB3);
    assert_eq!(hex_to_byte(b"B4").unwrap(), 0xB4);
    assert_eq!(hex_to_byte(b"B5").unwrap(), 0xB5);
    assert_eq!(hex_to_byte(b"B6").unwrap(), 0xB6);
    assert_eq!(hex_to_byte(b"B7").unwrap(), 0xB7);
    assert_eq!(hex_to_byte(b"B8").unwrap(), 0xB8);
    assert_eq!(hex_to_byte(b"B9").unwrap(), 0xB9);
    assert_eq!(hex_to_byte(b"BA").unwrap(), 0xBA);
    assert_eq!(hex_to_byte(b"BB").unwrap(), 0xBB);
    assert_eq!(hex_to_byte(b"BC").unwrap(), 0xBC);
    assert_eq!(hex_to_byte(b"BD").unwrap(), 0xBD);
    assert_eq!(hex_to_byte(b"BE").unwrap(), 0xBE);
    assert_eq!(hex_to_byte(b"BF").unwrap(), 0xBF);
    assert_eq!(hex_to_byte(b"C0").unwrap(), 0xC0);
    assert_eq!(hex_to_byte(b"C1").unwrap(), 0xC1);
    assert_eq!(hex_to_byte(b"C2").unwrap(), 0xC2);
    assert_eq!(hex_to_byte(b"C3").unwrap(), 0xC3);
    assert_eq!(hex_to_byte(b"C4").unwrap(), 0xC4);
    assert_eq!(hex_to_byte(b"C5").unwrap(), 0xC5);
    assert_eq!(hex_to_byte(b"C6").unwrap(), 0xC6);
    assert_eq!(hex_to_byte(b"C7").unwrap(), 0xC7);
    assert_eq!(hex_to_byte(b"C8").unwrap(), 0xC8);
    assert_eq!(hex_to_byte(b"C9").unwrap(), 0xC9);
    assert_eq!(hex_to_byte(b"CA").unwrap(), 0xCA);
    assert_eq!(hex_to_byte(b"CB").unwrap(), 0xCB);
    assert_eq!(hex_to_byte(b"CC").unwrap(), 0xCC);
    assert_eq!(hex_to_byte(b"CD").unwrap(), 0xCD);
    assert_eq!(hex_to_byte(b"CE").unwrap(), 0xCE);
    assert_eq!(hex_to_byte(b"CF").unwrap(), 0xCF);
    assert_eq!(hex_to_byte(b"D0").unwrap(), 0xD0);
    assert_eq!(hex_to_byte(b"D1").unwrap(), 0xD1);
    assert_eq!(hex_to_byte(b"D2").unwrap(), 0xD2);
    assert_eq!(hex_to_byte(b"D3").unwrap(), 0xD3);
    assert_eq!(hex_to_byte(b"D4").unwrap(), 0xD4);
    assert_eq!(hex_to_byte(b"D5").unwrap(), 0xD5);
    assert_eq!(hex_to_byte(b"D6").unwrap(), 0xD6);
    assert_eq!(hex_to_byte(b"D7").unwrap(), 0xD7);
    assert_eq!(hex_to_byte(b"D8").unwrap(), 0xD8);
    assert_eq!(hex_to_byte(b"D9").unwrap(), 0xD9);
    assert_eq!(hex_to_byte(b"DA").unwrap(), 0xDA);
    assert_eq!(hex_to_byte(b"DB").unwrap(), 0xDB);
    assert_eq!(hex_to_byte(b"DC").unwrap(), 0xDC);
    assert_eq!(hex_to_byte(b"DD").unwrap(), 0xDD);
    assert_eq!(hex_to_byte(b"DE").unwrap(), 0xDE);
    assert_eq!(hex_to_byte(b"DF").unwrap(), 0xDF);
    assert_eq!(hex_to_byte(b"E0").unwrap(), 0xE0);
    assert_eq!(hex_to_byte(b"E1").unwrap(), 0xE1);
    assert_eq!(hex_to_byte(b"E2").unwrap(), 0xE2);
    assert_eq!(hex_to_byte(b"E3").unwrap(), 0xE3);
    assert_eq!(hex_to_byte(b"E4").unwrap(), 0xE4);
    assert_eq!(hex_to_byte(b"E5").unwrap(), 0xE5);
    assert_eq!(hex_to_byte(b"E6").unwrap(), 0xE6);
    assert_eq!(hex_to_byte(b"E7").unwrap(), 0xE7);
    assert_eq!(hex_to_byte(b"E8").unwrap(), 0xE8);
    assert_eq!(hex_to_byte(b"E9").unwrap(), 0xE9);
    assert_eq!(hex_to_byte(b"EA").unwrap(), 0xEA);
    assert_eq!(hex_to_byte(b"EB").unwrap(), 0xEB);
    assert_eq!(hex_to_byte(b"EC").unwrap(), 0xEC);
    assert_eq!(hex_to_byte(b"ED").unwrap(), 0xED);
    assert_eq!(hex_to_byte(b"EE").unwrap(), 0xEE);
    assert_eq!(hex_to_byte(b"EF").unwrap(), 0xEF);
    assert_eq!(hex_to_byte(b"F0").unwrap(), 0xF0);
    assert_eq!(hex_to_byte(b"F1").unwrap(), 0xF1);
    assert_eq!(hex_to_byte(b"F2").unwrap(), 0xF2);
    assert_eq!(hex_to_byte(b"F3").unwrap(), 0xF3);
    assert_eq!(hex_to_byte(b"F4").unwrap(), 0xF4);
    assert_eq!(hex_to_byte(b"F5").unwrap(), 0xF5);
    assert_eq!(hex_to_byte(b"F6").unwrap(), 0xF6);
    assert_eq!(hex_to_byte(b"F7").unwrap(), 0xF7);
    assert_eq!(hex_to_byte(b"F8").unwrap(), 0xF8);
    assert_eq!(hex_to_byte(b"F9").unwrap(), 0xF9);
    assert_eq!(hex_to_byte(b"FA").unwrap(), 0xFA);
    assert_eq!(hex_to_byte(b"FB").unwrap(), 0xFB);
    assert_eq!(hex_to_byte(b"FC").unwrap(), 0xFC);
    assert_eq!(hex_to_byte(b"FD").unwrap(), 0xFD);
    assert_eq!(hex_to_byte(b"FE").unwrap(), 0xFE);
    assert_eq!(hex_to_byte(b"FF").unwrap(), 0xFF);
}

#[test]
fn hex_to_byte_invalid() {
    assert!(match hex_to_byte(b"z") {
        None => true,
        _    => false
    });

    assert!(match hex_to_byte(b"9g") {
        None => true,
        _    => false
    });

    assert!(match hex_to_byte(b"z3") {
        None => true,
        _    => false
    });
}

#[test]
#[should_panic]
fn hex_to_byte_panic() {
    assert!(match hex_to_byte(b"") {
        None => true,
        _    => false
    });
}
