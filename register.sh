eval "$(cat .env <(echo) <(declare -x))"

ENDPOINT="https://discord.com/api/v10"

DISCORD_APPLICAION_ID=$(
    curl -H "Authorization: Bot $DISCORD_TOKEN" \
        "$ENDPOINT/applications/@me" | \
        jq -r ".id"
)

echo $DISCORD_APPLICAION_ID

curl -X PUT $ENDPOINT/applications/$DISCORD_APPLICAION_ID/commands \
    -H "Authorization: Bot $DISCORD_TOKEN" \
    -H "Content-Type: application/json" \
    -d @./commands.locks.json