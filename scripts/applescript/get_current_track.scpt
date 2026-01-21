-- Get current track information from Apple Music
-- Returns: trackName|artistName|albumName|duration|position|playerState

tell application "Music"
    if player state is not stopped then
        set trackName to name of current track
        set artistName to artist of current track
        set albumName to album of current track
        set trackDuration to duration of current track
        set playerPosition to player position
        set playerState to player state as string
        
        -- Format: "trackName|artistName|albumName|duration|position|state"
        return trackName & "|" & artistName & "|" & albumName & "|" & trackDuration & "|" & playerPosition & "|" & playerState
    else
        return "||||||stopped"
    end if
end tell
