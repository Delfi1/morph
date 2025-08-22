#defs
$text_color = Hsla{ hue:45 saturation:1.0 lightness:0.5 alpha:1.0 }

#scenes
"menu"
    FlexNode{
        width: 65px 
        height: 40px
        flex_direction: Row 
        justify_main: Center 
        justify_cross: Center
    }
    "test_button"
        TextLine{ text: "Test" }
        TextLineColor($text_color)
    

