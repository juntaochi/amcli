-- Control Apple Music playback

on run argv
    set command to item 1 of argv
    
    tell application "Music"
        if command is "play" then
            play
        else if command is "pause" then
            pause
        else if command is "toggle" then
            playpause
        else if command is "next" then
            next track
        else if command is "previous" then
            previous track
        else if command is "stop" then
            stop
        end if
    end tell
end run
