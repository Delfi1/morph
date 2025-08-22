#defs
$text_colour = Hsla{hue:45 saturation:1.0 lightness:0.5 alpha:1.0}

#scenes
"scene"
    AbsoluteNode{left:40%}
    "cell"
        Animated<BackgroundColor>{
            idle:#FF0000
            hover:Hsla{ hue:120 saturation:1.0 lightness:0.50 alpha:1.0 }
            press:Hsla{ hue:240 saturation:1.0 lightness:0.50 alpha:1.0 }
        }
        NodeShadow{color:#FF0000 spread_radius:10px blur_radius:5px}
        "text"
            TextLine{text:"Hello, World!, I am writing using cobweb "}

"exit_button"
    TextLine{text:"Exit"}
    TextLineColor($text_colour)
    