# Name the skills a prompt matches. Portable awk: no gensub, no length(array).
#
#   awk -F'\t' -f route-skills.awk skill-routes.tsv skill-index.tsv - < hook-payload.json
#
# skill-routes.tsv: hand-written intents. skill-index.tsv: the table scripts/skill_router.py
# builds. Standard input: the hook's JSON, from which only the prompt field is read, so a
# path in `cwd` can never match a skill.
# Prints one `slug<TAB>blurb` line per match, best first, then `STANDING` if the prompt
# gives a standing instruction. Prints nothing when nothing clears the bar, which is the
# ordinary answer to "Continue".

function stem(w) {
    if (length(w) > 5 && w ~ /ies$/) return substr(w, 1, length(w) - 3) "y"
    if (length(w) > 5 && w ~ /ing$/) return substr(w, 1, length(w) - 3)
    if (length(w) > 4 && w ~ /ed$/) return substr(w, 1, length(w) - 2)
    if (length(w) > 4 && w ~ /es$/) return substr(w, 1, length(w) - 2)
    if (length(w) > 3 && w ~ /s$/ && w !~ /ss$/) return substr(w, 1, length(w) - 1)
    return w
}

BEGIN {
    # STOPWORDS and BOILERPLATE, in that order; test_hooks.py holds this equal to skill_router.py.
    n = split("a about above after again all also am an and any are as at be been before being but by can could did do does doing done down each even every few for from further get gets getting had has have having he her here hers him his how i if in into is it its itself just let lets like make makes making may me might more most much must my need needs no nor not now of off on once one only or other our out over own please same she should so some still such than that the their them then there these they this those through to too under until up us use used uses using very was way we were what when where which while who why will with without would yes yet you your covers skill already someone something anything nothing thing things whatever whoever instead rather across another whether because around first second third next last once new old real long short small big good bad right wrong want wanted going back keep look looks show shows give gives take takes", words, " ")
    for (i = 1; i <= n; i++) stop[words[i]] = 1
    MIN_SCORE = 120     # one rare pair in common can be chance: this takes two, or a slug's own words
    MAX_NAMED = 2
    MAX_CHARS = 4000    # past this it is a pasted log or a document, not a request
    MAX_STANDING = 400  # a standing order is one sentence the person typed
    MIN_WORDS = 4       # "Yes, deploy it" answers a question; the earlier turn was routed
}

FILENAME ~ /skill-routes[.]tsv$/ {
    if ($0 !~ /^#/ && NF >= 2) { routes++; route_slug[routes] = $1; route_re[routes] = $2 }
    next
}

FILENAME ~ /skill-index[.]tsv$/ {
    count++
    slug[count] = $1
    blurb[count] = $2
    blurb_of[$1] = $2
    entries = split($3, pairs, ",")
    for (i = 1; i <= entries; i++) {
        split(pairs[i], kv, "=")
        weight[count, kv[1]] = kv[2] + 0
    }
    next
}

{ payload = payload $0 " " }

END {
    if (!match(payload, /"prompt"[ ]*:[ ]*"([^"\\]|\\.)*"/)) exit 0
    text = substr(payload, RSTART, RLENGTH)
    sub(/^"prompt"[ ]*:[ ]*"/, "", text)
    if (text ~ /^[ ]*</) exit 0     # a task notification or scheduled run, not a person asking
    if (text ~ /\[(watch|climb)\]|plugin sent a message/) exit 0     # a mod's own prompt
    if (length(text) > MAX_CHARS) exit 0
    quoted = (text ~ /\\"/)         # text the person quoted is not an order from the person
    text = tolower(text)
    gsub(/\\u2019/, "", text)        # the curly apostrophe, when the JSON escapes it
    gsub(/\\[nrt]/, " ", text)
    gsub(/\\./, " ", text)
    gsub(/\047/, "", text)
    gsub(/\342\200\231/, "", text)  # the curly apostrophe, as UTF-8 bytes
    gsub(/<[^>]*>/, " ", text)

    standing = (!quoted && length(text) <= MAX_STANDING &&
                text ~ /(^|[.!;] *)(also,? *)?(from now on|from here on)|for every (update|release|build)|all future|never forget|dont forget|do not forget|remind to always|always (do|build|run|use|deploy|release)/)

    gsub(/[^a-z0-9%]+/, " ", text)
    n = split(text, words, " ")
    if (n < MIN_WORDS) { if (standing) print "STANDING"; exit 0 }
    padded = " " text " "

    # A continuation brief mentions everything its session touched. Only resuming is the request.
    if (padded ~ /^ background from the session that started /) MAX_NAMED = 1

    named = 0
    for (r = 1; r <= routes && named < MAX_NAMED; r++) {
        if (padded ~ route_re[r] && !(route_slug[r] in done)) {
            done[route_slug[r]] = 1
            named++
            print route_slug[r] "\t" blurb_of[route_slug[r]]
        }
    }

    previous = ""
    for (i = 1; i <= n; i++) {
        w = words[i]
        if (length(w) < 3 || (w in stop)) { previous = ""; continue }
        current = stem(w)
        if (previous != "") seen[previous "_" current] = 1
        previous = current
    }
    while (named < MAX_NAMED) {
        pick = 0
        best = MIN_SCORE - 1
        for (s = 1; s <= count; s++) {
            if (slug[s] in done) continue
            total = 0
            for (t in seen) if ((s, t) in weight) total += weight[s, t]
            if (total > best) { best = total; pick = s }
        }
        if (!pick) break
        done[slug[pick]] = 1
        named++
        print slug[pick] "\t" blurb[pick]
    }
    if (standing) print "STANDING"
}
