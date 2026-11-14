pub struct Logo {
    pub lines: &'static [&'static str],
    pub hue: f64,
}

const ARCH: &[&str] = &[
    "      /\\       ",
    "     /  \\      ",
    "    /    \\     ",
    "   /      \\    ",
    "  /   ,,   \\   ",
    " /   |  |   \\  ",
    "/ -''    ''-\\",
];

const CACHYOS: &[&str] = &[
    "         .-------------------------. ",
    "       .////////////////////////////. ",
    "      /////           ////////////// ",
    "     /////            ",
    "    /////       .////. ",
    "   /////       //////// ",
    "  /////         '////' ",
    " /////           ////////////// ",
    " ////////////////////////////' ",
    "  '////////////////////////' ",
    "    '////////////////////' ",
];

const ARTIX: &[&str] = &[
    "         /\\          ",
    "        /  \\         ",
    "       / `'.,\\       ",
    "      /   ,.' `. \\   ",
    "     /                \\ ",
    "    /                  \\ ",
    "   /                    \\ ",
];

const ENDEAVOUR: &[&str] = &[
    "                     ./o. ",
    "                   ./sssso- ",
    "                  `:osssssss+- ",
    "                `:+sssssssssso/. ",
    "              `-/ossssssssssssso/. ",
    "            .-/+sssssssssssssssso+:` ",
    "          `-:/+sssssssssssssssssso+/. ",
    "        `.://osssssssssssssssssssso++-` ",
    "       .://+ssssssssssssssssssssssso++/:` ",
    "     .:///ossssssssssssssssssssssssso++/:` ",
    "   ` :////ssssssssssssssssssssssssssso+++/. ",
    "  `-////+ssssssssssssssssssssssssssso++++-` ",
    " `..-+oosssssssssssssssssssssssso+++++/` ",
    "   ./++++++++++++++++++++++++++++++/:. ",
    "  `:::::::::::::::::::::::::::::---`` ",
];

const MANJARO: &[&str] = &[
    "||||||||||||||  ||||||",
    "||||||||||||||  ||||||",
    "||||||          ||||||",
    "||||||  ||||||  ||||||",
    "||||||  ||||||  ||||||",
    "||||||  ||||||  ||||||",
    "||||||  ||||||  ||||||",
    "||||||  ||||||  ||||||",
    "||||||  ||||||  ||||||",
    "||||||  ||||||  ||||||",
];

const GARUDA: &[&str] = &[
    "                     .%;888:8898898: ",
    "                   x;XxXB%89b8:b8%b88: ",
    "                .8Xxd                8X:. ",
    "              .8Xx;                    8x:. ",
    "            .tt8x          .d            x88; ",
    "         .@8x8;          .db:              xx@; ",
    "       ,tSXX°          .bbbbbbbbbbbbbbbbbbbB8x@; ",
    "     .SXxx            bBBBBBBBBBBBBBBBBBBBbSBX8; ",
    "   ,888S                                     pd! ",
    "  8X88/                                       q ",
    "  GBB. ",
    "   x%88        d888@8@X@X@X88X88X88X888888x;. ",
    "     x888%     888@8@X@X@X88X88X88X8888888X8x; ",
    "       :888X   888@8@X@X@X88X88X88X8888888X8x ",
    "         :@88x. ",
];

const DEBIAN: &[&str] = &[
    r#"    ,met$$$$$gg.    "#,
    r#"  ,g$$$$$$$$$$$$$$$P. "#,
    r#",g$$P"        "Y$$.""#,
    r#"',$$P'               `$$$."#,
    r#"',$$P ,ggs.     `$$b:"#,
    r#" `d$$'  ,$P"'   .    $$$"#,
    r#" $$P  d$'     ,    $$P"#,
    r#" $$: $$.   - ,d$$'"#,
    r#" $$;      Y$b._   _,d$P'"#,
    r#" Y$$.    `.`"Y$$$$P"'"#,
    r#" `$$b      "-.__"#,
    r#"  `Y$$b"#,
    r#"   `Y$."#,
    r#"    `$$b."#,
    r#"      `Y$$b."#,
    r#"         `"Y$b._"#,
];

const MX_LINUX: &[&str] = &[
    "MMMMMMMMMMMMMMMMMMMMMMMMMMMMMMNMMMMMMMMM",
    "MMMMMMMMMMNs..yMMMMMMMMMMMMMm: +NMMMMMMM",
    "MMMMMMMMMN+    :mMMMMMMMMMN+ `-dMMMMMMMM",
    "MMMMMMMMMMMs.` oNMMMMMMh-   `sNMMMMMMMM",
    "MMMMMMMMMMMMNo`    -yMMMN+ `:dMMMMMMMMMM",
    "MMMMMMMMMMMMMMm: +mh-` oNMMMMMMMMMMM",
    "MMMMMMMMMMMMMMMNy `.`   -hMMMMMMMMMMMMM",
    "MMMMMMMMMMMMMMMNy + `:NMMMMMMMMMMMMM",
    "MMMMMMMMMMMMMMm:` yMd-   `sNMMMMMMMMMMM",
    "MMMMMMMMMMMMNo`    :mMMMN+ `:dMMMMMMMMMM",
    "MMMMMMMMMMMs. .sNMMMMMMh-` oNMMMMMMMM",
    "MMMMMMMMMN+    `+NMMMMMMMMMN+`   -dMMMMMMM",
    "MMMMMMMNy.   -hMMMMMMMMMMMMMm:   `yMMMMMM",
    "MMMMMNh:` `oNMMMMMMMMMMMMMMMMs`   -hMMMM",
    "MMMms.    `:dMMMMMMMMMMMMMMMMMMMN+`   .sMM",
    "MMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMM",
];

const MINT: &[&str] = &[
    "             ...-:::::-...",
    "          .-MMMMMMMMMMMMMMM-.",
    "      .-MMMM `..-:::::::-..` MMMM-.",
    "    .:MMMM.:MMMMMMMMMMMMMMM:.MMMM:.",
    "   -MMM-M---MMMMMMMMMMMMMMMMMMM.MMM-",
    "  `:MMM:MM`   :MMMM:....::-...-MMMM:MMM:`",
    " :MMM:MMM`   :MM: `````` :MMM:MMM:",
    ".MMM.MMMM `:MM. -MM. .MM-` MMMM.MMM.",
    ":MMM:MMMM `:MM. -MM- .MM:` MMMM-MMM:",
    ":MMM:MMMM `:MM. -MM- .MM:` MMMM:MMM:",
    ":MMM:MMMM `:MM. -MM- .MM:` MMMM-MMM:",
    ".MMM.MMMM `:MM:--:MM:--:MM:` MMMM.MMM.",
    " :MMM:MMM-   `-MMMMMMMMMMMM-`   -MMM-MMM:",
    "  :MMM:MMM: :MMM:MMM:",
    "   .MMM.MMMM:--------------:MMMM.MMM.",
    "     '-MMMM.-MMMMMMMMMMMMMMM-.MMMM-'",
    "       '.-MMMM `--:::::--` MMMM-.'",
    "            '-MMMMMMMMMMMMM-'",
    "                `-:::::-`",
];

const UBUNTU: &[&str] = &[
    "            .-/+oossssoo+/-. ",
    "         `:+ssssssssssssssssss+:` ",
    "      -+ssssssssssssssssssyyssss+-",
    "    .ossssssssssssssssssdMMMNysssso. ",
    "   /ssssssssssshdmmNNmmyNMMMMhssssss/ ",
    "  +ssssssssshmydMMMMMMMNddddyssssssss+ ",
    " /sssssssshNMMMyhhyyyyhmNMMMNhssssssss/ ",
    ".ssssssssdMMMNhsssssssssshNMMMdssssssss. ",
    "+sssshhhyNMMNyssssssssssssyNMMMysssssss+ ",
    "ossyNMMMNyMMhsssssssssssssshmmmhssssssso ",
    "+sssshhhyNMMNyssssssssssssyNMMMysssssss+ ",
    ".ssssssssdMMMNhsssssssssshNMMMdssssssss. ",
    " /sssssssshNMMMyhhyyyyhdNMMMNhssssssss/ ",
    "  +sssssssssdmydMMMMMMMMddddyssssssss+ ",
    "   /ssssssssssshdmNNNNmyNMMMMhssssss/ ",
    "    .ossssssssssssssssssdMMMNysssso. ",
    "      -+sssssssssssssssssyyyssss+-",
    "         `:+ssssssssssssssssss+:` ",
    "            .-/+oossssoo+/-. ",
];

const POP: &[&str] = &[
    "             ///////////// ",
    "         ///////////////////// ",
    "      /////// 767//////////////// ",
    "    //////7676767676 ////////////// ",
    "   /////76767//7676767////////////// ",
    "  /////767676/// 76767/////////////// ",
    " ///////767676///76767.///7676 /////// ",
    "/////////767676//76767///767676//////// ",
    "//////////76767676767////76767///////// ",
    "///////////76767676//////7676////////// ",
    "////////////,7676,///////767/////////// ",
    "/////////////*7676///////76//////////// ",
    "///////////////7676//////////////////// ",
    " ///////////////7676///767//////////// ",
    "  //////////////////////'//////////// ",
    "   //////.7676767676767676767,////// ",
    "    /////767676767676767676767///// ",
    "      /////////////////////////// ",
    "         ///////////////////// ",
    "             ///////////// ",
];

const ZORIN: &[&str] = &[
    "         `osssssssssssssssssssso` ",
    "       .osssssssssssssssssssssso. ",
    "      .+oooooooooooooooooooooooo+. ",
    " ",
    " ",
    "   `::::::::::::::::::::::. `:. ",
    "  `+ssssssssssssssssss+:.` `.:+ss+` ",
    ".ossssssssssssssso/.        `-+osssssso.",
    "sssssssssssssso/-` `-/osssssssssss",
    "` ossssssssss+--/osssssssssssso `",
    "` :+sssssso. -/osssssssssssss+: `",
    "` :::-.:+osssssssssssssss+ `",
    "",
    "",
    " .+oooooooooooooooooooooooo+.",
    " .osssssssssssssssssssssso.",
    "` osssssssssssssssssssso` ",
];

const ELEMENTARY: &[&str] = &[
    "         eeeeeeeeeeeeeeeee ",
    "      eeeeeeeeeeeeeeeeeeeeeee ",
    "    eeeee  eeeeeeeeeeee   eeeee ",
    "  eeee   eeeee       eee     eeee ",
    " eeee   eeee          eee     eeee ",
    "eee    eee            eee       eee ",
    "eee   eee            eee        eee ",
    "ee    eee           eeee       eeee ",
    "ee    eee         eeeee      eeeeee ",
    "ee    eee       eeeee      eeeee ee ",
    "eee   eeee   eeeeee      eeeee  eee ",
    "eee    eeeeeeeeee     eeeeee    eee ",
    " eeeeeeeeeeeeeeeeeeeeeeee    eeeee ",
    "  eeeeeeee eeeeeeeeeeee      eeee ",
    "    eeeee                 eeeee ",
    "      eeeeeeeeeeeeeeeeeeeeeee ",
    "         eeeeeeeeeeeeeeeee ",
];

const KDE_NEON: &[&str] = &[
    "              `..---+/---..` ",
    "          `---.``   ``` .---. `",
    " .--.`     ``-:-.`     ",
    " `:/:   `.----//----.`   :/-",
    " .:.    `---`   `--.`   .:`",
    " .:`     `--`     .:-    `:.",
    " `/      `:.    `.-::-.`  -:`  `/`",
    " /.      /.    `:++++++++:`  .:  ./",
    "`/      .:   `++++++++++++`  /`  `/`",
    "/.      /`   ++++++++++++++  ./  ./",
    "/.      /`   ++++++++++++++  ./  ./",
    "`/      .:   `++++++++++++`  /`  `/`",
    " /.      /.    `:++++++++:`  .:  ./",
    " `/      `:.    `.-::-.`  -:`  `/`",
    " .:`     `--`     .:-    `:.",
    " .:.    `---`   `--.`   .:`",
    " `:/:   `.----//----.`   :/-",
    " .-:.``     `-:-.`     ",
    "` ---.   `.---.` ",
    "              `..---+/---..` ",
];

const KALI: &[&str] = &[
    ".............. ",
    "            ..,;:ccc,. ",
    "          ......''';lxO. ",
    ".....''''..........,:ld; ",
    "           .';;;:::;,,.x, ",
    "      ..'''.            0Xxoc:,.  ... ",
    "  ....                ,ONkc;,;cokOdc',. ",
    " .                   OMo           ':ddo. ",
    "                    dMc               :OO; ",
    "                    0M.                 .:o. ",
    "                    ;Wd ",
    "                     ;XO, ",
    "                       ,d0Odlc;,.. ",
    "                           ..',;:cdOOd::,. ",
    "                                    .:d;.':;. ",
    "                                       'd,  .' ",
    "                                         ;l   .. ",
    "                                          .o ",
    "                                            c ",
    "                                            .' ",
    "                                             . ",
];

const PARROT: &[&str] = &[
    "   `:oho/-` `-/oho:` ",
    " `+MMMMMMMMmy/` `/ymMMMMMMMM+` ",
    " `dMMMMMMMMMMMMms:` `:smMMMMMMMMMMMMd` ",
    " .mMMMMMMMMMMMMMMNddNMMMMMMMMMMMMMMm. ",
    "   `sNMMMMMMMMMMMMMMMMMMMMMMMMMMMMNs` ",
    "     `/dMMMMMMMMMMMMMMMMMMMMMMMMd/` ",
    "       .+dMMMMMMMMMMMMMMMMMMd+. ",
    "           `/yNMMMMMMMMMMNy/` ",
    "              `+NMMMMMMN+` ",
    "                `dMMMMd` ",
    "                 +MM+` ",
    "                  `` ",
];

const DEEPIN: &[&str] = &[
    "             ............ ",
    "         .';;;;;.       .,;,. ",
    "      .,;;;;;;;.       ';;;;;;;. ",
    "    .;::::::::'     .,::;;,''''',. ",
    "   ,'.::::::::    .;;'.          '; ",
    "  ;'  'cccccc,  ,' :: '..        .: ",
    " ,,    :ccccc. ;: .c, '' :.       ;; ",
    ".l.     cllll' ., .lc  :; .l'     lc. ",
    ".c       :lllc  ;cl:  .l' .ll.   :'c. ",
    ".l        'looc. .   ,o:  'oo'  c, l. ",
    ".o.         .:ool::coc'  .ooo' o. .o. ",
    " ::            .....   .;dddo ;c  ;: ",
    "  l:                  .lddddo;l' :l ",
    "   ;o,               ;xdddddod: ,o; ",
    "    'dxl,.       .,oxdddddddo'.lxd' ",
    "      'oxxolccccloxdddddddo;.lxx' ",
    "         .';codddddddoc;'. ",
];

const DEVUAN: &[&str] = &[
    "   ..,,;;;::;,.. ",
    "            `':ddd;:,.",
    "` 'dPPd:,. ",
    "                      `:b$$b` . ",
    "                        'P$$$d `",
    " .$$$$$` ",
    "                         ;$$$$$P ",
    "                      .:P$$$$$$ `",
    " .,:b$$$$$$$;'",
    " .,:dP$$$$$$$$b:'",
    " .,:;db$$$$$$$$$$Pd'` ",
    " ,db$$$$$$$$$$$$$$b:' `",
    ":$$$$$$$$$$$$b:'` ",
    "  `$$$$$bd:''` ",
    "    `'''` ",
];

const FEDORA: &[&str] = &[
    "             .',;::::;,'. ",
    "         .';:cccccccccccc:;,. ",
    "      .;cccccccccccccccccccccc;. ",
    "    .:cccccccccccccccccccccccccc:. ",
    "  .;ccccccccccccc;.:dddl:.;ccccccc;. ",
    " .:ccccccccccccc;OWMKOOXMWd;ccccccc:. ",
    ".:ccccccccccccc;KMMc;cc;xMMc:ccccccc:. ",
    ",cccccccccccccc;MMM.;cc;;WW::cccccccc, ",
    ":cccccccccccccc;MMM.;cccccccccccccccc: ",
    ":ccccccc;oxOOOo;MMM0OOk.;cccccccccccc: ",
    "cccccc:0MMKxdd:;MMMkddc.;cccccccccccc; ",
    "ccccc:XM0';cccc;MMM.;cccccccccccccccc' ",
    "ccccc;MMo;ccccc;MMW.;ccccccccccccccc; ",
    "ccccc;0MNc.ccc.xMMd:ccccccccccccccc; ",
    "cccccc;dNMWXXXWM0::cccccccccccccc:, ",
    "cccccccc;.:odl:.;cccccccccccccc:,. ",
    ":cccccccccccccccccccccccccccc:'. ",
    ".:cccccccccccccccccccccc:;,.. ",
    "  '::cccccccccccccc::;,. ",
];

const NOBARA: &[&str] = &[
    "        .',;::::;,'. ",
    "     .';cccccccccccc;'. ",
    "   .;cccccccccccccccccc;. ",
    "  ;cccccccccccccccccccccc; ",
    " ;ccccccc;.;cccc;.;ccccccc; ",
    ":ccccccc;  ;cccc;  ;ccccccc: ",
    ":ccccccc:  :cccc:  :ccccccc: ",
    ";cccccccc;.;cccc;.;cccccccc; ",
    " ;cccccccccccccccccccccccc; ",
    "  ;cccccccc;.;cccccccccc; ",
    "   .;cccccc;  ;ccccccc;. ",
    "     .';ccc;  ;ccc;'. ",
    "        .',;::;'. ",
];

const BAZZITE: &[&str] = &[
    "         .,ldkOOOOOkdl,. ",
    "      .;d00000000000000d;. ",
    "    .o0000000d,..;d0000000o. ",
    "   .k000000k'      'k000000k. ",
    "   k000000k.   ..   .k000000k ",
    "   0000000.   k00k   .0000000 ",
    "   0000000.  '0000'  .0000000 ",
    "   k000000k.  '..`  .k000000k ",
    "   .k000000k,      ,k000000k. ",
    "    .o0000000d:..:d0000000o. ",
    "      .;d00000000000000d;. ",
    "         .,ldkOOOOOkdl,. ",
];

const ALMALINUX: &[&str] = &[
    "         'c:. ",
    "        lkkkx, ..       ..   ,cc, ",
    "        okkkk:ckkx'  .lxkkx.okkkkd ",
    "        .:llcokkx'  :kkkxkko:xkkd, ",
    "      .xkkkkdood:  ;kx,  .lkxlll; ",
    "       xkkx.       xk'     xkkkkk: ",
    "       'xkx.       xd      .....,. ",
    "      .. :xkl'     :c      ..''.. ",
    "    .dkx'  .:ldl:'. '  ':lollldkkx; ",
    "  .''lkkko'                     ckkkc. ",
    "'xkkkd:kkd.       ..  ;'        :kkxo. ",
    ",xkkkd;kk'      ,d;    ld.   ':dkd::cc, ",
    " .,,.;xkko'.';lxo.      dx,  :kkk'xkkkkc ",
    "     'dkkkkkxo:.        ;kx  .kkk:;xkkd. ",
    "       .....   .;dk:.   lkk.  :;, ",
    "             :kkkkkkkdoxkkx ",
    "              ,c,,;;;:xkkd. ",
    "                ;kkkkl... ",
    "                ;kkkkl ",
    "                 ,od; ",
];

const ROCKY: &[&str] = &[
    "           wglZDz + . ",
    "        mBSSSSSSSSSSSSBmw ",
    "      mSSSSSSSSSSSSSSSSSSSm ",
    "    dSSSSSSSSSSSSSSSSSSSSSSSb ",
    "   dSSSSSSSSSSSSSSSSSSSSSSSSSb ",
    "  dSSSSSSSSSSSSSSSSSSSSSSSSSSSb ",
    "  SSSSSSSSSSSSSSSmSSSSSSSSSSSSS ",
    "  SSSSSSSSSSSSSm'  `mSSSSSSSSSSS",
    " SSSSSSSSSSSm'` mSSSSSSSSS ",
    "   SSSSSSSSm'    _     `mSSSSSS",
    " SSSSSm' _mSm_` mSSS ",
    "      `SS' _mSSSSSm_` S' ",
    "            mSSSSSSSSSm ",
    "          `SSSSSSSSSSSSS' ",
];

const CENTOS: &[&str] = &[
    "                 .. ",
    "               .PLTJ. ",
    "               < > < > < > < > ",
    "     KKSSV' 4KKK LJ KKKL.'VSSKK ",
    "     KKV' 4KKKKK LJ KKKKAL 'VKK ",
    "     V' ' 'VKKKK LJ KKKKV' ' 'V ",
    "     .4MA.' 'VKK LJ KKV' '.4Mb. ",
    "   . KKKKKA.' 'V LJ V' '.4KKKKK . ",
    " .4D KKKKKKKA.'' LJ ''.4KKKKKKK FA. ",
    " <QDD ++++++++++++  ++++++++++++ GFD > ",
    " 'VD KKKKKKKK'.. LJ ..'KKKKKKKK FV' ",
    "   ' VKKKKK'. .4 LJ K. .'KKKKKV ' ",
    "      'VK'. .4KK LJ KKA. .'KV' ",
    "     A. . .4KKKK LJ KKKKA. . .4 ",
    "     KKA. 'KKKKK LJ KKKKK' .4KK ",
    "     KKSSA. VKKK LJ KKKV .4SSKK ",
    "               < > < > < > < > ",
    "               'MKKM' ",
    "                 '' ",
];

const RHEL: &[&str] = &[
    "           .MMM..:MMMMMMM ",
    "          MMMMMMMMMMMMMMMMMM ",
    "          MMMMMMMMMMMMMMMMMMMM. ",
    "         MMMMMMMMMMMMMMMMMMMMMM ",
    "        ,MMMMMMMMMMMMMMMMMMMMMM: ",
    "        MMMMMMMMMMMMMMMMMMMMMMMM ",
    "  .MMMM'  MMMMMMMMMMMMMMMMMMMMMM ",
    " MMMMMM     `MMMMMMMMMMMMMMMMMMMM.",
    "MMMMMMMM MMMMMMMMMMMMMMMMMM .",
    "MMMMMMMMM.` MMMMMMMMMMMMM' MM. ",
    "MMMMMMMMMMM.                     MMMM ",
    " `MMMMMMMMMMMMM. ,MMMMM.",
    "` MMMMMMMMMMMMMMMMM.          ,MMMMMMMM. ",
    "    MMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMM ",
    "      MMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMM: ",
    "         MMMMMMMMMMMMMMMMMMMMMMMMMMMMMM ",
    "            `MMMMMMMMMMMMMMMMMMMMMMMM: ",
    "                ``MMMMMMMMMMMMMMMMM' ",
];

const OPENSUSE: &[&str] = &[
    "           .;ldkO0000Okdl;. ",
    "       .;d00xl:^''''''^:ok00d;. ",
    "     .d00l'                'o00d. ",
    "   .d0Kd'  Okxol:;,.          :O0d. ",
    "  .OKKKK0kOKKKKKKKKKKOxo:,      lKO. ",
    " ,0KKKKKKKKKKKKKKKK0P^,,,^dx:    ;00, ",
    ".OKKKKKKKKKKKKKKKKk'.oOPPb.'0k.   cKO. ",
    ":KKKKKKKKKKKKKKKKK: kKx..dd lKd   'OK: ",
    "dKKKKKKKKKKKOx0KKKd ^0KKKO' kKKc   dKd ",
    "dKKKKKKKKKKKK;.;oOKx,..^..;kKKK0.  dKd ",
    ":KKKKKKKKKKKK0o;...^cdxxOK0O/^^'  .0K: ",
    " kKKKKKKKKKKKKKKK0x;,,......,;od  lKk ",
    " '0KKKKKKKKKKKKKKKKKKKKK00KKOo^  c00' ",
    "  'kKKKOxddxkOO00000Okxoc;''   .dKk' ",
    "    l0Ko.                    .c00l' ",
    "     'l0Kk:.              .;xK0l' ",
    "        'lkK0xl:;,,,,;:ldO0kl' ",
    "            '^:ldxkkkkxdl:^' ",
];

const ALPINE: &[&str] = &[
    "       .hddddddddddddddddddddddh. ",
    "      :dddddddddddddddddddddddddd: ",
    "     /dddddddddddddddddddddddddddd/ ",
    "    +dddddddddddddddddddddddddddddd+ ",
    "   `sdddddddddddddddddddddddddddddddds` ",
    "  `ydddddddddddd++hdddddddddddddddddddy` ",
    ".hddddddddddd+ +ddddh:-sdddddddddddh. ",
    "hdddddddddd+ +y:    .sddddddddddh ",
    "ddddddddh+ // . `-sddddddddd",
    "ddddddh+` `/hddh/` `:s- -sddddddd",
    "ddddh+` `/+/dddddh/` `+s- -sddddd",
    "ddh+` `/o`  :dddddddh/ oy-    .yddd ",
    "hdddyo+ohddyosdddddddddho+oydddy++ohdddh ",
    ".hddddddddddddddddddddddddddddddddddddh. ",
    "  `yddddddddddddddddddddddddddddddddddy` ",
    "   `sdddddddddddddddddddddddddddddddds` ",
    "    +dddddddddddddddddddddddddddddd+ ",
    "     /dddddddddddddddddddddddddddd/ ",
    "      :dddddddddddddddddddddddddd: ",
    "       .hddddddddddddddddddddddh. ",
];

const GENTOO: &[&str] = &[
    "         -/oyddmdhs+:. ",
    "     -odNMMMMMMMMNNmhy+-`",
    " -yNMMMMMMMMMMMNNNmmdhy+-",
    "` omMMMMMMMMMMMMNmdmmmmddhhy/ `",
    " omMMMMMMMMMMMNhhyyyohmdddhhhdo` ",
    ".ydMMMMMMMMMMdhs++so/smdddhhhhdm+ `",
    " oyhdmNMMMMMMMNdyooydmddddhhhhyhNd.",
    " :oyhhdNNMMMMMMMNNNmmdddhhhhhyymMh",
    " .:+sydNMMMMMNNNmmmdddhhhhhhmMmy",
    " /mMMMMMMNNNmmmdddhhhhhmMNhs:",
    "` oNMMMMMMMNNNmmmddddhhdmMNhs+ `",
    "` sNMMMMMMMMNNNmmmdddddmNMmhs/. ",
    " /NMMMMMMMMNNNNmmmdddmNMNdso: `",
    "+MMMMMMMNNNNNmmmmdmNMNdso/-",
    "yMMNNNNNNNmmmmmNNMmhs+/-` ",
    "/hMMNNNNNNNNMNdhs++/-`",
    "` /ohdmmddhys+++/:. `",
    "` -//////:--. ",
];

const NIXOS: &[&str] = &[
    "          ::::.    ':::::     ::::' ",
    "          ':::::    ':::::.  ::::' ",
    "            :::::     '::::.::::: ",
    "      .......:::::..... ::::::. ",
    "     ::::::::::::::::::. :::::. ",
    "    ':::::::::::::::::::: :::::. ",
    "               .:::::'    ':::::. ",
    "              .:::::'      ':::::. ",
    "    .:::::.  .:::::'        ':::::. ",
    "   .:::::::.:::::'            ':::::. ",
    "  .::::::::::::::..............::::::. ",
    "   ':::::::::::::::::::::::::::::::::' ",
    "    ':::::'  ':::::.     ':::::. ",
    "     .::::'    ':::::.    ':::::. ",
    "    .::::'      ':::::.:::::. ",
];

const VOID: &[&str] = &[
    "                 .;=====;. ",
    "             .=+==++=++=+=+===;. ",
    "             -=+++=+===+=+=+++++= ",
    "        .     -=:``      `--==+=++==.",
    " _vi,`             --+=++++: ",
    "      .uvnvi.        .        -==+==+. ",
    "     .vvnvnI `.;==|==;. :|=||=|.",
    "+QmQQmAllow: .:|||||||: :|||||||",
    "` QMMMMMMQQ/     :|||||||:     :||||||| ",
    "  `QQQQQQQ: '=|==|='` |=||=|' ",
    "    `+QQQQm. -=+===+'",
    "` +QQmQoi;,.        .,;+++++=' ",
    "          `-+QQQQmQQQooo+++++===+'",
    "` --===+++++===--' ",
];

const SOLUS: &[&str] = &[
    "            -``````````` ",
    "           `-+/------------.` ",
    "       .---:mNo---------------. ",
    "     .-----yMMMy:---------------. ",
    "    `------oMMMMMm/----------------` ",
    "  .------/MMMMMMMN+----------------. ",
    " .------/NMMMMMMMMm-+/--------------. ",
    " `------/NMMMMMMMMMN-:mh/-------------` ",
    ".-----/NMMMMMMMMMMM:-+MMd//oso/:-----. ",
    "-----/NMMMMMMMMMMMM+--mMMMh::smMmyo:--",
    "----+NMMMMMMMMMMMMMo--yMMMMNo-:yMMMMd. ",
    ".--oMMMMMMMMMMMMMMMy--yMMMMMMh:-yMMMy `",
    "` -sMMMMMMMMMMMMMMMMh--dMMMMMMMd:/Ny+ `",
    "` -/+osyhhddmmNNNMMo--mMMMMMMMm:```` ",
    "    `----------------++----------` ",
    "      `-------------------------` ",
    "        `---------------------` ",
    "           `---------------` ",
];

const SLACKWARE: &[&str] = &[
    "                   ::::::: ",
    "             ::::::::::::::::::: ",
    "          ::::::::::::::::::::::::: ",
    "        ::::::::cllcccccllllllll:::::: ",
    "     :::::::::lc               dc::::::: ",
    "    ::::::::cl   clllccllll    oc::::::::: ",
    "   :::::::::o   lc::::::::co   oc:::::::::: ",
    "  ::::::::::o    cccclc:::::clcc:::::::::::: ",
    "  :::::::::::lc        cclccclc::::::::::::: ",
    " ::::::::::::::lcclcc          lc:::::::::::: ",
    " ::::::::::cclcc:::::lccclc     oc::::::::::: ",
    " ::::::::::o    l::::::::::l    lc::::::::::: ",
    "  :::::cll:o     clcllcccll     o::::::::::: ",
    "  :::::occ:o                  clc::::::::::: ",
    "   ::::ocl:ccslclccclclccclclc::::::::::::: ",
    "    :::oclcccccccccccccllllllllllllll:::: ",
    "     ::lcc1lcccccccccccccccccccccccco:::: ",
    "       :::::::::::::::::::::::::::::::: ",
    "         :::::::::::::::::::::::::::: ",
    "            :::::::::::::::::::::: ",
    "                 :::::::::::: ",
];

const MAGEIA: &[&str] = &[
    "               *@@@@@ ",
    "            *@@@@@@@@@ ",
    "               @@@@ ",
    "                ** ",
    "            .o@@@@@o. ",
    "          .@@@@@@@@@@@. ",
    "         .@@@@@@@@@@@@@. ",
    "          '@@@@@@@@@@@' ",
    "            '@@@@@@@' ",
    "        .@@@@@@   @@@@@@. ",
    "      .@@@@@@@@   @@@@@@@@. ",
    "     .@@@@@@@@@   @@@@@@@@@. ",
    "     @@@@@@@@@@@@@@@@@@@@@@@ ",
    "      '@@@@@@@@@@@@@@@@@@@' ",
    "        '@@@@@@@@@@@@@@@' ",
    "           ''@@@@@@@'' ",
];

const STEAMOS: &[&str] = &[
    "              .,od88888bo,. ",
    "          .,d888888888888888b,. ",
    "        ,d888888888888888888888b, ",
    "      ,888888888888888888888888888, ",
    "     d88888888888888888888888888888b ",
    "    888888888888888888888P'   'Y88888 ",
    "    8888888888888888888P  .888.  Y888 ",
    "    8888888888888888888  8888888  888 ",
    "    '888888888888888888. '88888' .88P ",
    "     '8888888888888888P'   `''' .88P",
    " 'Y8888888888P' .d8P'",
    " 'Y88888P' .d88P'",
    "` 'Y888888888888P'' ",
];

const TUX: &[&str] = &[
    "     .--.      ",
    "    |o_o |     ",
    "    |:_/ |     ",
    "   //   \\ \\    ",
    "  (|     | )   ",
    " /'_    /`\\   ",
    " _ )=( /   ",
];

fn match_id(id: &str) -> Option<Logo> {
    match id {
        "cachyos" => Some(Logo {
            lines: CACHYOS,
            hue: 168.0,
        }),
        "arch" | "archlinux" | "archarm" | "arcolinux" | "arcolinuxd" | "arcolinuxb"
        | "archcraft" | "bluestar" | "omarchy" | "rebornos" => Some(Logo {
            lines: ARCH,
            hue: 200.0,
        }),
        "artix" => Some(Logo {
            lines: ARTIX,
            hue: 192.0,
        }),
        "endeavouros" => Some(Logo {
            lines: ENDEAVOUR,
            hue: 270.0,
        }),
        "manjaro" | "manjaro-arm" | "biglinux" | "mabox" => Some(Logo {
            lines: MANJARO,
            hue: 145.0,
        }),
        "garuda" => Some(Logo {
            lines: GARUDA,
            hue: 185.0,
        }),
        "steamos" => Some(Logo {
            lines: STEAMOS,
            hue: 220.0,
        }),
        "debian" | "raspbian" | "antix" | "q4os" | "sparky" | "sparkylinux" | "tails"
        | "minios" | "pureos" | "bunsenlabs" => Some(Logo {
            lines: DEBIAN,
            hue: 340.0,
        }),
        "mx" | "mxlinux" => Some(Logo {
            lines: MX_LINUX,
            hue: 202.0,
        }),
        "devuan" => Some(Logo {
            lines: DEVUAN,
            hue: 272.0,
        }),
        "kali" => Some(Logo {
            lines: KALI,
            hue: 205.0,
        }),
        "parrot" => Some(Logo {
            lines: PARROT,
            hue: 182.0,
        }),
        "deepin" | "uos" | "openkylin" => Some(Logo {
            lines: DEEPIN,
            hue: 200.0,
        }),
        "linuxmint" | "mint" | "lmde" => Some(Logo {
            lines: MINT,
            hue: 132.0,
        }),
        "pop" => Some(Logo {
            lines: POP,
            hue: 186.0,
        }),
        "zorin" => Some(Logo {
            lines: ZORIN,
            hue: 198.0,
        }),
        "elementary" => Some(Logo {
            lines: ELEMENTARY,
            hue: 210.0,
        }),
        "neon" => Some(Logo {
            lines: KDE_NEON,
            hue: 175.0,
        }),
        "ubuntu" | "kubuntu" | "xubuntu" | "lubuntu" | "ubuntu-mate" | "ubuntustudio"
        | "ubuntukylin" | "edubuntu" | "anduinos" | "tuxedo" | "lite" | "linuxlite"
        | "peppermint" | "bodhi" | "winux" | "nitrux" | "pardus" => Some(Logo {
            lines: UBUNTU,
            hue: 20.0,
        }),
        "fedora" => Some(Logo {
            lines: FEDORA,
            hue: 215.0,
        }),
        "nobara" => Some(Logo {
            lines: NOBARA,
            hue: 230.0,
        }),
        "bazzite" => Some(Logo {
            lines: BAZZITE,
            hue: 262.0,
        }),
        "almalinux" => Some(Logo {
            lines: ALMALINUX,
            hue: 195.0,
        }),
        "rocky" => Some(Logo {
            lines: ROCKY,
            hue: 145.0,
        }),
        "centos" => Some(Logo {
            lines: CENTOS,
            hue: 280.0,
        }),
        "rhel" | "redhat" | "ol" | "oracle" => Some(Logo {
            lines: RHEL,
            hue: 355.0,
        }),
        "opensuse"
        | "opensuse-tumbleweed"
        | "opensuse-leap"
        | "opensuse-slowroll"
        | "sles"
        | "suse" => Some(Logo {
            lines: OPENSUSE,
            hue: 115.0,
        }),
        "mageia" | "openmandriva" | "pclinuxos" => Some(Logo {
            lines: MAGEIA,
            hue: 210.0,
        }),
        "alpine" | "postmarketos" => Some(Logo {
            lines: ALPINE,
            hue: 210.0,
        }),
        "gentoo" | "funtoo" | "calculate" => Some(Logo {
            lines: GENTOO,
            hue: 275.0,
        }),
        "nixos" => Some(Logo {
            lines: NIXOS,
            hue: 208.0,
        }),
        "void" => Some(Logo {
            lines: VOID,
            hue: 140.0,
        }),
        "solus" => Some(Logo {
            lines: SOLUS,
            hue: 214.0,
        }),
        "slackware" | "puppy" => Some(Logo {
            lines: SLACKWARE,
            hue: 224.0,
        }),
        _ => None,
    }
}

pub fn for_distro(id: &str, id_like: Option<&str>) -> Logo {
    let lower = id.to_ascii_lowercase();
    if let Some(logo) = match_id(&lower) {
        return logo;
    }
    if let Some(like) = id_like {
        for token in like.split_whitespace() {
            if let Some(logo) = match_id(&token.to_ascii_lowercase()) {
                return logo;
            }
        }
    }
    Logo {
        lines: TUX,
        hue: 50.0,
    }
}
