#!/usr/bin/env python3
"""Generates grouped Weir rules (each with 15+ tests) for phrase-level fixes.

Run from the repo root: python3 scripts/editor-review/gen_rules.py
Each group becomes one directory under weir_rules, which is one public rule in Settings.
"""
import os, re

BASE = "harper-core/src/linting/weir_rules"

NOUN_FRAMES = [
    "The {p} is clear.", "I think the {p} was important.", "We discussed the {p} yesterday.",
    "Nobody noticed the {p}.", "She wrote about the {p} again.", "{P} came up in the meeting.",
    "They talked about {p} for hours.", "It was {p}, as expected.", "He thought about {p} for a while.",
    "There was no {p} here.", "Her {p} surprised us.", "Mention the {p} to them.",
]
VERB_FRAMES = [
    "We will {p} the files.", "Please {p} the pieces.", "They want to {p} soon.", "I need to {p} now.",
    "Can you {p} them?", "{P} the two groups.", "He asked us to {p} everything.", "She likes to {p} early.",
    "We should {p} before noon.", "You can {p} later.", "Try to {p} today.", "They will {p} next week.",
]
ADJ_FRAMES = [
    "It is {p}.", "That was {p}.", "The result was {p}.", "She felt {p}.", "This seems {p}.",
    "{P} indeed.", "We found it {p}.", "He called it {p}.", "They are {p}.", "Everything looked {p}.",
    "The plan is {p}.", "Honestly, it is {p}.",
]
CLEAN = ["The weather is nice today.", "Please send the report by Friday.", "She enjoys long walks.",
         "We met on Tuesday afternoon.", "Thanks for the update."]

def cap(s): return s[0].upper() + s[1:]

def camel(s): return "".join(w.capitalize() for w in re.split(r"[^A-Za-z0-9]+", s) if w)

def write_rule(group, name, expr, message, description, kind, becomes, strategy, tests, allows):
    d = os.path.join(BASE, group)
    os.makedirs(d, exist_ok=True)
    out = [f"expr main {expr}", "", f'let message "{message}"', f'let description "{description}"',
           f'let kind "{kind}"', f'let becomes "{becomes}"']
    if strategy: out.append(f'let strategy "{strategy}"')
    out.append("")
    for a, b in tests: out.append(f'test "{a}" "{b}"')
    out.append("")
    for a in allows: out.append(f'allows "{a}"')
    open(os.path.join(d, name + ".weir"), "w").write("\n".join(out) + "\n")

def phrase_rule(group, bad, good, frames, message, description, kind="Redundancy", expr=None, extra_allows=()):
    p_bad, p_good = bad, good
    tests = []
    for f in frames:
        tests.append((f.format(p=p_bad, P=cap(p_bad)), f.format(p=p_good, P=cap(p_good))))
    allows = [f.format(p=p_good, P=cap(p_good)) for f in frames[:5]] + CLEAN[:3] + list(extra_allows)
    words = " ".join(bad.split())
    write_rule(group, camel(bad), expr or f"({words})", message, description, kind, good, "MatchCase", tests, allows)

# ---- Redundancy -------------------------------------------------------------
G = "RedundantPhrases"
for bad, good, frames in [
    ("past history", "history", NOUN_FRAMES), ("free gift", "gift", NOUN_FRAMES),
    ("end result", "result", NOUN_FRAMES), ("final outcome", "outcome", NOUN_FRAMES),
    ("close proximity", "proximity", NOUN_FRAMES), ("general consensus", "consensus", NOUN_FRAMES),
    ("new innovation", "innovation", NOUN_FRAMES), ("old adage", "adage", NOUN_FRAMES),
    ("sum total", "total", NOUN_FRAMES), ("true fact", "fact", NOUN_FRAMES),
    ("unexpected surprise", "surprise", NOUN_FRAMES), ("usual custom", "custom", NOUN_FRAMES),
    ("mutual cooperation", "cooperation", NOUN_FRAMES), ("basic fundamentals", "fundamentals", NOUN_FRAMES),
    ("revert back", "revert", VERB_FRAMES), ("reply back", "reply", VERB_FRAMES),
    ("join together", "join", VERB_FRAMES), ("merge together", "merge", VERB_FRAMES),
    ("combine together", "combine", VERB_FRAMES), ("collaborate together", "collaborate", VERB_FRAMES),
    ("cooperate together", "cooperate", VERB_FRAMES), ("repeat again", "repeat", VERB_FRAMES),
    ("return back", "return", VERB_FRAMES),
    ("completely unanimous", "unanimous", ADJ_FRAMES), ("absolutely essential", "essential", ADJ_FRAMES),
    ("completely finished", "finished", ADJ_FRAMES),
]:
    phrase_rule(G, bad, good, frames, f"`{bad}` is redundant. Use `{good}`.",
                f"Removes the redundant word in `{bad}`.")

phrase_rule(G, "in spite of the fact that", "although", NOUN_FRAMES[:0] + [
    "{P}, we left.", "We stayed {p}, it rained.", "{P} it was late, he called.",
    "I came {p} I was tired.", "She won {p} the odds were against her.", "{P} they argued, they stayed.",
    "He smiled {p} he was sad.", "We went out {p} it snowed.", "{P} the delay, we won.",
    "They finished {p} it was hard.", "{P} the rain, we played.", "I stayed {p} it hurt.",
], "`in spite of the fact that` is wordy. Use `although`.", "Shortens `in spite of the fact that`.", "Style")
phrase_rule(G, "until such time as", "until", [
    "We wait {p} the vote.", "Stay {p} the end.", "{P} the sun sets, rest.", "Hold on {p} we call.",
    "Keep going {p} we finish.", "They stayed {p} help came.", "I will wait {p} you arrive.",
    "She slept {p} noon.", "{P} dawn, we walked.", "He read {p} midnight.", "We sat {p} it ended.", "Pause {p} later."
], "`until such time as` is wordy. Use `until`.", "Shortens `until such time as`.", "Style")
phrase_rule(G, "subsequent to", "after", [
    "We met {p} the meeting.", "{P} lunch, we left.", "They arrived {p} the storm.", "I called {p} dinner.",
    "She left {p} the show.", "He slept {p} the trip.", "We talked {p} the game.", "{P} the call, I left.",
    "They ate {p} the lecture.", "I rested {p} work.", "Come {p} noon.", "We paid {p} delivery."
], "`subsequent to` is stiff. Use `after`.", "Replaces `subsequent to` with `after`.", "Style")
phrase_rule(G, "HIV virus", "HIV", NOUN_FRAMES, "`HIV` already stands for `human immunodeficiency virus`, so `virus` is redundant.",
            "Removes `virus` after HIV.")

# ---- Context homophones -----------------------------------------------------
def ctx_rule(group, name, expr, becomes, message, description, pos, neg, kind="Spelling", strategy="MatchCase"):
    write_rule(group, name, expr, message, description, kind, becomes, strategy, pos, neg)

H = "ContextHomophones"
ctx_rule(H, "BreakTime", "<([(take a), (takes a), (took a), (taken a), (taking a), (give a), (lunch), (coffee), (tea), (spring), (winter), (summer), (commercial), (bathroom), (smoke)] brake), (brake)>",
  "break", "Did you mean `break` (a pause)?", "A pause or rest is a `break`; a `brake` stops a vehicle.",
  [(f"Let's take a brake {t}.", f"Let's take a break {t}.") for t in ["now","soon","here","today","later"]] +
  [("We took a brake.", "We took a break."), ("She is taking a brake.", "She is taking a break."),
   ("Lunch brake starts at noon.", "Lunch break starts at noon."), ("Coffee brake time.", "Coffee break time."),
   ("Summer brake is here.", "Summer break is here."), ("Winter brake starts soon.", "Winter break starts soon."),
   ("He takes a brake daily.", "He takes a break daily."), ("They need a spring brake.", "They need a spring break."),
   ("Bathroom brake please.", "Bathroom break please."), ("A commercial brake followed.", "A commercial break followed.")],
  ["Press the brake pedal.", "The brake light is on.", "She hit the brake hard.", "Take a break.", "He needs a coffee break.", "Check the brake fluid."])
ctx_rule(H, "BrakePedal", "<(break [pedal, pads, fluid, light, lights, caliper, line, lines, disc, discs, rotor, rotors]), (break)>",
  "brake", "Did you mean `brake` (the vehicle part)?", "The vehicle part is a `brake`; a `break` is a pause or fracture.",
  [(f"Press the break {w}.", f"Press the brake {w}.") for w in ["pedal","pads","fluid","light","lights"]] +
  [("Check the break pads.", "Check the brake pads."), ("The break fluid leaked.", "The brake fluid leaked."),
   ("Replace the break pedal.", "Replace the brake pedal."), ("Break pads wear out.", "Brake pads wear out."),
   ("The break lines are old.", "The brake lines are old."), ("A break disc cracked.", "A brake disc cracked."),
   ("The break caliper stuck.", "The brake caliper stuck."), ("Inspect the break rotor.", "Inspect the brake rotor."),
   ("The break lights failed.", "The brake lights failed."), ("A break line leaked.", "A brake line leaked.")],
  ["Take a break.", "Press the brake pedal.", "A bone break takes weeks.", "Break the glass.", "We break for lunch."])
ctx_rule(H, "HitTheBrakes", "<([hit, hits, hitting, slam, slams, slammed, slamming, pump, pumped, pumping] the breaks), (breaks)>",
  "brakes", "Did you mean `brakes`?", "Vehicle brakes are `brakes`; `breaks` are pauses or fractures.",
  [(f"{v} the breaks.", f"{v} the brakes.") for v in ["Hit","Slam","Pump"]] +
  [("He hit the breaks hard.", "He hit the brakes hard."), ("She slammed the breaks.", "She slammed the brakes."),
   ("They are slamming the breaks.", "They are slamming the brakes."), ("I hit the breaks in time.", "I hit the brakes in time."),
   ("We pumped the breaks.", "We pumped the brakes."), ("He hits the breaks often.", "He hits the brakes often."),
   ("Slam the breaks now.", "Slam the brakes now."), ("She hit the breaks suddenly.", "She hit the brakes suddenly."),
   ("Pumping the breaks helps.", "Pumping the brakes helps."), ("He is hitting the breaks.", "He is hitting the brakes.")],
  ["He hit the brakes hard.", "We take the breaks seriously.", "Coffee breaks are short.", "She hit the ball.", "Slam the door."])
ctx_rule(H, "Heard", "<([I, we, you, he, she, they] herd [(a noise), (a sound), (a voice), (the noise), (the sound), (the news), (a rumor), (that), (about), (from), (nothing), (something), (anything), (someone), (it)]), (herd)>",
  "heard", "Did you mean `heard`?", "A `herd` is a group of animals; `heard` is the past tense of `hear`.",
  [(f"I herd {w}.", f"I heard {w}.") for w in ["a noise","a sound","a voice","the news","a rumor","that","nothing","something","someone","from them"]] +
  [("We herd the noise.", "We heard the noise."), ("She herd about it.", "She heard about it."),
   ("They herd a sound.", "They heard a sound."), ("He herd that story.", "He heard that story."), ("You herd the news.", "You heard the news.")],
  ["A herd of cattle crossed.", "I heard a noise.", "They herd the sheep.", "We herd the cattle home.", "The herd moved on.", "She heard the news."])
ctx_rule(H, "PlainSight", "<(plane [sight, text, English, language, vanilla, truth, view, speaking]), (plane)>",
  "plain", "Did you mean `plain`?", "The adjective meaning simple or clear is `plain`; a `plane` flies or is flat.",
  [(f"It was in plane {w}.", f"It was in plain {w}.") for w in ["sight","view"]] +
  [("Use plane text.", "Use plain text."), ("Write in plane English.", "Write in plain English."),
   ("Plain vanilla ice cream.", "Plain vanilla ice cream."), ("The plane truth hurts.", "The plain truth hurts."),
   ("Hidden in plane sight.", "Hidden in plain sight."), ("A plane language policy.", "A plain language policy."),
   ("Save as plane text.", "Save as plain text."), ("He is plane speaking.", "He is plain speaking."),
   ("Plane English works best.", "Plain English works best."), ("The plane view is clear.", "The plain view is clear."), ("It was plane sight.", "It was plain sight.")],
  ["The plane landed.", "Hidden in plain sight.", "Write in plain English.", "A plane flew over.", "We boarded the plane."])
ctx_rule(H, "WebSite", "<([web, construction, building, job, historic, archaeological, landfill] cite), (cite)>",
  "site", "Did you mean `site`?", "A location or web page is a `site`; to `cite` is to quote a source.",
  [(f"The {w} cite was busy.", f"The {w} site was busy.") for w in ["web","construction","building","job","historic"]] +
  [("Visit our web cite.", "Visit our web site."), ("The job cite opens today.", "The job site opens today."),
   ("A historic cite nearby.", "A historic site nearby."), ("The construction cite is loud.", "The construction site is loud."),
   ("The archaeological cite is old.", "The archaeological site is old."), ("The landfill cite closed.", "The landfill site closed."),
   ("Our web cite is down.", "Our web site is down."), ("The building cite is fenced.", "The building site is fenced."),
   ("A new job cite opened.", "A new job site opened."), ("The web cite loads fast.", "The web site loads fast.")],
  ["Cite your sources.", "The web site loads fast.", "Please cite the study.", "The construction site is loud.", "He will cite the law.", "You must cite them."])
ctx_rule(H, "CiteSources", "<([please, to, must, should, always, never, and, will, can, cannot, you, we, they] site [your, (the sources), sources, evidence, references, studies, research, (a source), (any sources), (all sources), (them properly)]), (site)>",
  "cite", "Did you mean `cite`?", "To quote or reference a source is to `cite` it; a `site` is a place.",
  [(f"Please site {w}.", f"Please cite {w}.") for w in ["your sources","evidence","references","studies","research","a source","all sources"]] +
  [("You must site your sources.", "You must cite your sources."), ("Always site evidence.", "Always cite evidence."),
   ("We should site research.", "We should cite research."), ("They will site studies.", "They will cite studies."),
   ("Never site sources badly.", "Never cite sources badly.")],
  ["Please cite your sources.", "The site has evidence.", "Visit the site for references.", "A site for research.", "The new site is live.", "Please cite the study."])
ctx_rule(H, "KnowNotNo", "<([to, don't, didn't, I, you, we, they] no [what, how, why, where, when, who, if, whether, about, everything, anything]), (no)>",
  "know", "Did you mean `know`?", "The verb is `know`; `no` is a negative word.",
  [(f"I want to no {w}.", f"I want to know {w}.") for w in ["what","how","why","where","when","who","if","whether","about it","everything"]] +
  [("I don't no what happened.", "I don't know what happened."), ("They didn't no why.", "They didn't know why."),
   ("We no how it works.", "We know how it works."), ("You no what I mean.", "You know what I mean."), ("I no if it is true.", "I know if it is true.")],
  ["I want to know what happened.", "No one knew why.", "There is no way to tell.", "He has no idea.", "No what, no how.", "To no avail."])
ctx_rule(H, "WhetherNotWeather", "<([know, knows, knew, wonder, wonders, wondering, ask, asks, asked, decide, decided, deciding, unsure, sure, see, check, depends, determine, debate, debating] weather [(or not), (to go), (to stay), (to leave)]), (weather)>",
  "whether", "Did you mean `whether`?", "To introduce a choice, use `whether`; `weather` is about the sky.",
  [("We don't know weather or not it works.", "We don't know whether or not it works."),
   ("I wonder weather or not to leave.", "I wonder whether or not to leave."), ("Decide weather to go.", "Decide whether to go."),
   ("Ask weather to stay.", "Ask whether to stay."), ("She asked weather or not it rains.", "She asked whether or not it rains."),
   ("It depends weather or not you try.", "It depends whether or not you try."), ("They debate weather to go.", "They debate whether to go."),
   ("Unsure weather to stay.", "Unsure whether to stay."), ("He knows weather or not it is true.", "He knows whether or not it is true."),
   ("Check weather or not it works.", "Check whether or not it works."), ("I'm unsure weather to leave.", "I'm unsure whether to leave."),
   ("We decided weather or not to go.", "We decided whether or not to go."), ("See weather or not it helps.", "See whether or not it helps."),
   ("They are deciding weather to stay.", "They are deciding whether to stay."), ("I know weather or not he did.", "I know whether or not he did.")],
  ["The weather is nice.", "Bad weather or not, we ride.", "The weather today is warm.", "What's the weather?", "Whether or not you agree, we go.", "I know the weather will turn."])
ctx_rule(H, "TooNotTo", "<([is, are, am, was, were, be, so, much, far, way, not] to [tired, late, early, much, many, big, small, hot, cold, fast, slow, long, short, busy, expensive, high, low, young, old, soon, hard, easy, heavy, loud, quiet, dark, bright]), (to)>",
  "too", "Did you mean `too` (excessively)?", "Use `too` to mean `excessively`; `to` is a preposition or infinitive marker.",
  [(f"It is to {w}.", f"It is too {w}.") for w in ["tired","late","early","much","big","small","hot","cold","fast","slow","long","short"]] +
  [("He was to busy.", "He was too busy."), ("They are to expensive.", "They are too expensive."), ("We were to young.", "We were too young.")],
  ["It is too late.", "From hot to cold.", "I went to bed.", "He wants to be early.", "Go from fast to slow.", "She came to hard terms."])
ctx_rule(H, "YourNotYoure", "<([for, to, with, from, of, about, in, on, at, by] you're [help, time, patience, support, understanding, email, message, attention, order, feedback, business, reply, own, family, team, friend, friends, house, home, work, job, car, name, account]), (you're)>",
  "your", "Did you mean `your` (belonging to you)?", "Use `your` for possession; `you're` means `you are`.",
  [(f"Thanks for you're {w}.", f"Thanks for your {w}.") for w in ["help","time","patience","support","understanding","email","message","attention","order","feedback","business","reply"]] +
  [("Go with you're own plan.", "Go with your own plan."), ("Talk to you're team.", "Talk to your team."), ("Ask about you're family.", "Ask about your family.")],
  ["Thanks for your help.", "You're welcome for the help.", "Say hi to your team.", "I think you're right.", "We know you're busy.", "If you're ready, go."])
ctx_rule(H, "ItsOwn", "<(it's [own, owner, purpose, tail, paw, leash, color, colour]), (it's)>",
  "its", "Did you mean `its` (belonging to it)?", "Use `its` for possession; `it's` means `it is` or `it has`.",
  [(f"The dog wagged it's {w}.", f"The dog wagged its {w}.") for w in ["tail","paw"]] +
  [("It has it's own rules.", "It has its own rules."), ("The tool has it's purpose.", "The tool has its purpose."),
   ("The cat lost it's leash.", "The cat lost its leash."), ("It changed it's color.", "It changed its color."),
   ("It changed it's colour.", "It changed its colour."), ("The shop and it's owner left.", "The shop and its owner left."),
   ("A bird flapped it's tail.", "A bird flapped its tail."), ("The robot has it's own mind.", "The robot has its own mind."),
   ("Each tool has it's own use, own.", "Each tool has its own use, own."), ("The firm kept it's own books.", "The firm kept its own books."),
   ("The idea has it's own purpose.", "The idea has its own purpose.")],
  ["Its own rules apply.", "It's cold outside.", "It's a good day.", "The dog wagged its tail.", "I think it's true.", "It's been a while."])

# ---- Eggcorns ----------------------------------------------------------------
E = "Eggcorns"
ctx_rule(E, "TakenForGranted", "<([take, takes, took, taken, taking] [it, them, that, this, him, her] for granite), (granite)>",
  "granted", "Did you mean `for granted`?", "The idiom is `take for granted`, not `for granite`.",
  [(f"Don't {v} it for granite.", f"Don't {v} it for granted.") for v in ["take","took"]] +
  [("She took it for granite.", "She took it for granted."), ("He takes them for granite.", "He takes them for granted."),
   ("We took that for granite.", "We took that for granted."), ("I take this for granite.", "I take this for granted."),
   ("They took him for granite.", "They took him for granted."), ("Never take her for granite.", "Never take her for granted."),
   ("Don't take it for granite, friend.", "Don't take it for granted, friend."), ("Taking it for granite hurts.", "Taking it for granted hurts."),
   ("You take them for granite.", "You take them for granted."), ("We take that for granite daily.", "We take that for granted daily."),
   ("Sadly, he took it for granite.", "Sadly, he took it for granted.")],
  ["Don't take it for granted.", "The counter is granite.", "A granite wall stands.", "We take it as given.", "Granite is hard.", "She took it for a test."])
ctx_rule(E, "ShooIn", "(shoe-in)", "shoo-in", "Did you mean `shoo-in`?", "A sure winner is a `shoo-in`.",
  [(f"{n} is a shoe-in for the job.", f"{n} is a shoo-in for the job.") for n in ["She","He","Maria","Sam","Alex"]] +
  [("A shoe-in for the prize.", "A shoo-in for the prize."), ("He's a shoe-in to win.", "He's a shoo-in to win."),
   ("The shoe-in won again.", "The shoo-in won again."), ("They called her a shoe-in.", "They called her a shoo-in."),
   ("Such a shoe-in!", "Such a shoo-in!"), ("He was a shoe-in.", "He was a shoo-in."), ("Is she a shoe-in?", "Is she a shoo-in?"),
   ("Not a shoe-in at all.", "Not a shoo-in at all."), ("What a shoe-in.", "What a shoo-in."), ("A shoe-in, truly.", "A shoo-in, truly.")],
  ["She is a shoo-in for the job.", "Buy a new shoe.", "A shoe fits well.", "Tie your shoe.", "The shoe store opened.", "Shoes are on sale."])
ctx_rule(E, "ToeTheLine", "<(tow the line), (tow)>", "toe", "Did you mean `toe the line`?", "The idiom comes from standing with toes at a line: `toe the line`.",
  [(f"{n} must tow the line.", f"{n} must toe the line.") for n in ["You","He","She","We","They"]] +
  [("Tow the line or leave.", "Toe the line or leave."), ("He refused to tow the line.", "He refused to toe the line."),
   ("Please tow the line.", "Please toe the line."), ("They tow the line here.", "They toe the line here."),
   ("Learn to tow the line.", "Learn to toe the line."), ("I tow the line daily.", "I toe the line daily."),
   ("We always tow the line.", "We always toe the line."), ("Will you tow the line?", "Will you toe the line?"),
   ("She would tow the line.", "She would toe the line."), ("Staff tow the line.", "Staff toe the line.")],
  ["He must toe the line.", "We tow the car.", "Tow the boat home.", "The line is long.", "Toe the line."])
ctx_rule(E, "InTheBud", "<([nip, nips, nipped, nipping] [it, them, this, that, problems] in the butt), (butt)>", "bud", "Did you mean `nip it in the bud`?", "The idiom refers to pinching off a bud: `nip it in the bud`.",
  [(f"Nip {o} in the butt early.", f"Nip {o} in the bud early.") for o in ["it","them","this","that"]] +
  [("We must nip it in the butt.", "We must nip it in the bud."), ("Nip problems in the butt.", "Nip problems in the bud."),
   ("Let's nip this in the butt now.", "Let's nip this in the bud now."), ("Nip it in the butt, fast.", "Nip it in the bud, fast."),
   ("I nip it in the butt.", "I nip it in the bud."), ("She nipped it in the butt.", "She nipped it in the bud."),
   ("Try to nip it in the butt.", "Try to nip it in the bud."), ("Nip that in the butt today.", "Nip that in the bud today."),
   ("They nip them in the butt.", "They nip them in the bud."), ("We nip this in the butt together.", "We nip this in the bud together."),
   ("Please nip it in the butt.", "Please nip it in the bud.")],
  ["Nip it in the bud.", "He fell on his butt.", "A cigarette butt lay there.", "The bud opened.", "Nip the stem.", "A rose bud grew."])
ctx_rule(E, "ScotFree", "<(scott free), (scott free)>", "scot-free", "Did you mean `scot-free`?", "`Scot-free` means unpunished; it has nothing to do with a person named Scott.",
  [(f"{n} got off scott free.", f"{n} got off scot-free.") for n in ["He","She","They","We","Nobody"]] +
  [("He walked away scott free.", "He walked away scot-free."), ("She went scott free.", "She went scot-free."),
   ("They escaped scott free.", "They escaped scot-free."), ("Nobody gets off scott free.", "Nobody gets off scot-free."),
   ("He left scott free.", "He left scot-free."), ("Out scott free!", "Out scot-free!"), ("They went home scott free.", "They went home scot-free."),
   ("She got away scott free.", "She got away scot-free."), ("He is free, scott free.", "He is free, scot-free."), ("Off scott free.", "Off scot-free.")],
  ["He got off scot-free.", "Scott is free tonight.", "Scott went home.", "Call Scott soon.", "Scott and I left.", "Scott's free time ended."])
ctx_rule(E, "DogEatDog", "<(doggy dog world), (doggy dog world)>", "dog-eat-dog world", "Did you mean `dog-eat-dog world`?", "The saying is `dog-eat-dog world`.",
  [(f"{n} is a doggy dog world.", f"{n} is a dog-eat-dog world.") for n in ["It","This","That","Business"]] +
  [("It's a doggy dog world out there.", "It's a dog-eat-dog world out there."), ("A doggy dog world indeed.", "A dog-eat-dog world indeed."),
   ("Welcome to the doggy dog world.", "Welcome to the dog-eat-dog world."), ("Sales is a doggy dog world.", "Sales is a dog-eat-dog world."),
   ("It's such a doggy dog world.", "It's such a dog-eat-dog world."), ("This doggy dog world is hard.", "This dog-eat-dog world is hard."),
   ("A doggy dog world, truly.", "A dog-eat-dog world, truly."), ("Life is a doggy dog world.", "Life is a dog-eat-dog world."),
   ("It was a doggy dog world.", "It was a dog-eat-dog world."), ("In a doggy dog world, adapt.", "In a dog-eat-dog world, adapt.")],
  ["It's a dog-eat-dog world.", "The doggy bag was full.", "A dog world exists.", "My dog is fun.", "The doggy door is open.", "Dogs rule the world."])

# ---- Natural phrasing --------------------------------------------------------
N = "NaturalPhrasing"
ctx_rule(N, "AccordingToMe", "(according to [me, myself])", "in my opinion", "Use `in my opinion` or `I think`.",
  "`According to` introduces another source. For your own view, say `in my opinion` or `I think`.",
  [(f"{s} this is wrong.", f"{t} this is wrong.") for s, t in [("According to me,", "In my opinion,"), ("according to me,", "in my opinion,")]] +
  [("According to me, we should go.", "In my opinion, we should go."), ("According to me, it works.", "In my opinion, it works."),
   ("According to me this fails.", "In my opinion this fails."), ("According to me, he is right.", "In my opinion, he is right."),
   ("It is, according to me, fine.", "It is, in my opinion, fine."), ("According to me it is late.", "In my opinion it is late."),
   ("According to me, she won.", "In my opinion, she won."), ("According to me, the plan is good.", "In my opinion, the plan is good."),
   ("According to me, that is false.", "In my opinion, that is false."), ("According to me, try again.", "In my opinion, try again."),
   ("According to me, it's fine.", "In my opinion, it's fine."), ("According to me, we are done.", "In my opinion, we are done.")],
  ["According to the report, sales rose.", "According to her, it works.", "In my opinion, it works.", "According to Maria, we go.", "According to them, no.", "I think it works."], kind="Style")
for pron in ["you", "him", "her", "them", "us", "me"]:
    ctx_rule(N, "Explain" + pron.capitalize(), f"<([explain, explains, explained, explaining] {pron} [the, that, how, what, why, it, this, our, my, your]), ({pron})>", f"to {pron}",
      f"Use `explain to {pron}`.", "The verb `explain` needs `to` before the person who is told.",
      [(f"Let me explain {pron} {w}.", f"Let me explain to {pron} {w}.") for w in ["the plan","how it works","what happened","why it failed","it again","this clearly","our plan","my idea","your options","that now"]] +
      [(f"I will explain {pron} the rules.", f"I will explain to {pron} the rules."), (f"She explained {pron} how.", f"She explained to {pron} how."),
       (f"Please explain {pron} what is wrong.", f"Please explain to {pron} what is wrong."), (f"We can explain {pron} why.", f"We can explain to {pron} why."),
       (f"He explains {pron} the idea.", f"He explains to {pron} the idea.")],
      [f"Let me explain to {pron} the plan.", "Explain the plan.", "I will explain it later.", "She explained the rules.", "We explain how it works.", "Please explain why."], kind="Grammar", strategy=None)
ctx_rule(N, "DoTheNeedful", "<([do, does, did, doing] the needful), (the needful)>", "what is needed", "`Do the needful` sounds dated in American English.", "Say `do what is needed` or just describe the action.",
  [("Please do the needful.", "Please do what is needed."), ("Kindly do the needful.", "Kindly do what is needed."), ("Could you do the needful?", "Could you do what is needed?")] +
  [("Please do the needful today.", "Please do what is needed today."), ("I will do the needful.", "I will do what is needed."),
   ("He will do the needful soon.", "He will do what is needed soon."), ("Do the needful.", "Do what is needed."),
   ("They do the needful.", "They do what is needed."), ("We did the needful.", "We did what is needed."), ("He does the needful.", "He does what is needed."),
   ("Can you do the needful?", "Can you do what is needed?"), ("Kindly do the needful now.", "Kindly do what is needed now."),
   ("She can do the needful.", "She can do what is needed."), ("Let's do the needful.", "Let's do what is needed."),
   ("Please do the needful by noon.", "Please do what is needed by noon."), ("Do the needful quickly.", "Do what is needed quickly.")],
  ["Please do what is needed.", "Do the work.", "I will do it.", "He did the needed work.", "Do the dishes.", "We do the laundry."], kind="Style")

# ---- Agreement extras ---------------------------------------------------------
A = "AgreementExtras"
for hed, nouns in [("number", "people"), ("couple", "people"), ("handful", "people"), ("variety", "options")]:
    ctx_rule(A, f"A{hed.capitalize()}Of", f"<(a {hed} of * [is, was]), ([is, was])>", "are", "Use a plural verb here.",
      f"After `a {hed} of` plus a plural noun, the verb is plural.",
      [(f"A {hed} of {n} is waiting.", f"A {hed} of {n} are waiting.") for n in ["people","students","cars","files","guests","books"]] +
      [(f"a {hed} of men is here.", f"a {hed} of men are here."), (f"A {hed} of kids is late.", f"A {hed} of kids are late."),
       (f"A {hed} of ideas is ready.", f"A {hed} of ideas are ready.")] +
      [(f"A {hed} of things is wrong.", f"A {hed} of things are wrong."), (f"A {hed} of fans is here.", f"A {hed} of fans are here.")] +
      [(f"A {hed} of dogs is barking.", f"A {hed} of dogs are barking.")],
      [f"A {hed} of people are waiting.", f"The {hed} of people is large.", f"The {hed} of options is wide.", "A lot of water is gone.", "A group of students is waiting.", "A pair of shoes is here."],
      kind="Agreement")
for sub, wrong, right in [("were", "news were", "news was"), ("are", "news are", "news is")]:
    ctx_rule(A, f"News{sub.capitalize()}", f"<(news {sub}), ({sub})>", right.split()[-1], "`News` is singular.", "`News` takes a singular verb.",
      [(f"The news {sub} shocking.", f"The news {right.split()[-1]} shocking."), (f"News {sub} good today.", f"News {right.split()[-1]} good today.")] +
      [(f"The news {sub} {w}.", f"The news {right.split()[-1]} {w}.") for w in ["bad","good","clear","out","late","old","new","big","sad","true","false"]] +
      [(f"That news {sub} unexpected.", f"That news {right.split()[-1]} unexpected.")],
      ["The news is shocking.", "The news was good.", "No news is good news.", "I read the news.", "News travels fast.", "Good news, everyone."], kind="Agreement")
ctx_rule(A, "OneOfThePlural", "<(one of [the, my, our, your, his, her, their, these, those] * [are, were]), ([are, were])>", "is", "`One` is singular.", "In `one of the ...`, the verb agrees with `one`, so it is singular.",
  [(f"One of the {n} are late.", f"One of the {n} is late.") for n in ["reasons","students","cars","files","books"]] +
  [(f"One of my {n} were absent.", f"One of my {n} was absent.") for n in ["friends","cats"]] [:0] +
  [("One of the reasons are cost.", "One of the reasons is cost."), ("one of our players are hurt.", "one of our players is hurt."),
   ("One of their ideas are wrong.", "One of their ideas is wrong."), ("One of those things are broken.", "One of those things is broken."),
   ("One of these options are better.", "One of these options is better."), ("One of his books are missing.", "One of his books is missing."),
   ("One of her friends are here.", "One of her friends is here."), ("One of your answers are off.", "One of your answers is off."),
   ("One of my cats are sick.", "One of my cats is sick.")],
  ["One of the reasons is cost.", "One of my friends is here.", "Some of the reasons are clear.", "Two of the students are late.", "Both of the cars are red.", "One of them is gone."],
  strategy=None, kind="Agreement")

# ---- Usage extras -------------------------------------------------------------
U = "UsageExtras"
ctx_rule(U, "LedBy", "<([is, was, were, are, been, be, being] lead by), (lead)>", "led", "Use the past participle `led`.",
  "The past participle of `lead` is `led`; `lead` is the present form or a metal.",
  [(f"The team was lead by {n}.", f"The team was led by {n}.") for n in ["Maria","a veteran","the founder","two managers","an expert"]] +
  [("The company is lead by a former teacher.", "The company is led by a former teacher."), ("It was lead by example.", "It was led by example."),
   ("They were lead by guides.", "They were led by guides."), ("We are lead by data.", "We are led by data."),
   ("She has been lead by instinct.", "She has been led by instinct."), ("To be lead by fear is bad.", "To be led by fear is bad."),
   ("The march was lead by students.", "The march was led by students."), ("The project is lead by Sam.", "The project is led by Sam."),
   ("The group is lead by volunteers.", "The group is led by volunteers.")],
  ["The team was led by Maria.", "She will lead by example.", "We lead by listening.", "The lead by two points held.", "Lead pipes are old.", "He wants to lead by doing."], kind="Grammar")
ctx_rule(U, "UsedTo", "<([I, he, she] use to [be, live, have, own, love, hate, like, believe, smoke, drink, date, hang]), (use)>", "used", "Use `used to` for a past habit or state.",
  "A past habit is `used to`, not `use to`.",
  [(f"I use to {v} there.", f"I used to {v} there.") for v in ["be","live","have","own","love","hate","like","believe","smoke","drink"]] +
  [("He use to live here.", "He used to live here."), ("She use to be shy.", "She used to be shy."), ("I use to date him.", "I used to date him."),
   ("He use to own a car.", "He used to own a car."), ("She use to hang out.", "She used to hang out.")],
  ["I used to live there.", "I didn't use to smoke.", "Tools I use to build things.", "She uses it to read.", "We use to-do lists.", "I use it to live."], kind="Grammar")
ctx_rule(U, "SaidMeThat", "<(said [me, us, him, her, them] [that, to]), (said)>", "told", "Use `told` before the person addressed.",
  "You `say` something, but you `tell` someone something.",
  [(f"She said {p} that it worked.", f"She told {p} that it worked.") for p in ["me","us","him","her","them"]] +
  [(f"He said {p} to leave.", f"He told {p} to leave.") for p in ["me","us","him","her","them"]] +
  [("She said me that.", "She told me that."), ("They said us that.", "They told us that."), ("He said him that.", "He told him that."),
   ("I said her to go.", "I told her to go."), ("We said them that.", "We told them that.")],
  ["She told me that it worked.", "She said that it worked.", "He said her name.", "She said to him that it worked.", "They said so.", "He said it to me."], kind="Grammar")
ctx_rule(U, "BetweenYouAndMe", "<(between [you, him, her, them, us] and I), (I)>", "me", "Use `me` after `between`.",
  "After a preposition like `between`, use the object pronoun `me`.",
  [(f"Between {p} and I, it was dull.", f"Between {p} and me, it was dull.") for p in ["you","him","her","them","us"]] +
  [("Keep it between you and I.", "Keep it between you and me."), ("It is between him and I.", "It is between him and me."),
   ("A secret between you and I.", "A secret between you and me."), ("Between you and I, no.", "Between you and me, no."),
   ("The deal is between them and I.", "The deal is between them and me."), ("This stays between us and I.", "This stays between us and me."),
   ("Between her and I, it works.", "Between her and me, it works."), ("Just between you and I.", "Just between you and me."),
   ("Between you and I we agree.", "Between you and me we agree."), ("Peace between you and I.", "Peace between you and me.")],
  ["Between you and me, no.", "You and I agree.", "He and I left.", "Between us, it works.", "You and I are friends.", "Call you and I will go."], kind="Grammar", strategy="Exact")
nouns = ["people","items","cars","students","employees","customers","users","children","books","calls","emails","mistakes","errors","words","things","options","members","visitors","patients","players","kids","women","men"]
ctx_rule(U, "FewerNotLess", "<(less [" + ", ".join(nouns) + "]), (less)>", "fewer", "Use `fewer` with countable things.",
  "For things you can count, use `fewer`; use `less` for amounts.",
  [(f"Less {n} came.", f"Fewer {n} came.") for n in nouns[:12]] +
  [("There are less people today.", "There are fewer people today."), ("We have less cars now.", "We have fewer cars now."), ("Less students enrolled.", "Fewer students enrolled.")],
  ["Fewer people came.", "Less water is left.", "We have less time.", "Less than ten people came.", "He earns less money.", "There is less traffic."], kind="Usage")

# ---- Past participle after an adverb ("has already ate") ------------------------
P = "ParticipleAfterAdverb"
for past, pp in [("ate","eaten"),("went","gone"),("did","done"),("saw","seen"),("wrote","written"),("took","taken"),("drove","driven"),
                 ("spoke","spoken"),("broke","broken"),("began","begun"),("gave","given"),("rode","ridden"),("sang","sung"),("drank","drunk"),("swam","swum")]:
    adverbs = ["already","just","never","ever","still"]
    pos = []
    for i, (aux, adv) in enumerate([("has","already"),("have","just"),("had","never"),("has","just"),("have","already"),("had","already"),
                                    ("has","never"),("have","never"),("has","still"),("had","just"),("have","ever"),("has","ever")]):
        pos.append((f"She {aux} {adv} {past} it.", f"She {aux} {adv} {pp} it."))
    pos.append((f"They have already {past} twice.", f"They have already {pp} twice."))
    ctx_rule(P, "Has" + past.capitalize(), f"<([has, have, had] [already, just, never, ever, still] {past}), ({past})>", pp,
      f"Use the past participle `{pp}` here.", f"After `have`, `has` or `had`, the verb needs its past participle: `{pp}`.",
      pos,
      [f"She has already {pp} it.", f"They have just {pp} it.", f"She {past} it yesterday.", f"He already {past} it.", f"We never {past} it.", "She has finished."], kind="Grammar")

ctx_rule(A, "AndIIs", "<(* and I is), (is)>", "are", "Use a plural verb after `and I`.", "A compound subject such as `he and I` is plural.",
  [(f"{n} and I is ready.", f"{n} and I are ready.") for n in ["He","She","You","They","Sam","Maria","Dad","Mom","Ann","Tom"]] +
  [("he and I is friends.", "he and I are friends."), ("You and I is the team.", "You and I are the team."), ("Tom and I is going.", "Tom and I are going."),
   ("Ann and I is next.", "Ann and I are next."), ("Kim and I is here.", "Kim and I are here.")],
  ["He and I are ready.", "I am ready.", "You and I are friends.", "She and I agree.", "Sam and I went home.", "He and I were late."], kind="Agreement", strategy=None)
ctx_rule(A, "AndIWas", "<(* and I was), (was)>", "were", "Use a plural verb after `and I`.", "A compound subject such as `he and I` is plural.",
  [(f"{n} and I was late.", f"{n} and I were late.") for n in ["He","She","You","They","Sam","Maria","Dad","Mom","Ann","Tom"]] +
  [("he and I was friends.", "he and I were friends."), ("You and I was the team.", "You and I were the team."), ("Tom and I was going.", "Tom and I were going."),
   ("Ann and I was next.", "Ann and I were next."), ("Kim and I was here.", "Kim and I were here.")],
  ["He and I were late.", "I was late.", "You and I were friends.", "She and I agreed.", "Sam and I went home.", "He and I are ready."], kind="Agreement", strategy=None)
for subj in ["mathematics", "physics", "linguistics", "genetics"]:
    ctx_rule(A, subj.capitalize() + "Are", f"<({subj} are), (are)>", "is", f"`{subj.capitalize()}` is singular.", f"The field of study `{subj}` takes a singular verb.",
      [(f"{subj.capitalize()} are hard.", f"{subj.capitalize()} is hard."), (f"{subj.capitalize()} are fun.", f"{subj.capitalize()} is fun."), (f"{subj.capitalize()} are my major.", f"{subj.capitalize()} is my major.")] +
      [(f"I think {subj} are {w}.", f"I think {subj} is {w}.") for w in ["hard","fun","useful","vast","important","old","new","key","clear"]] +
      [(f"{subj.capitalize()} are taught here.", f"{subj.capitalize()} is taught here."), (f"Her {subj} are good.", f"Her {subj} is good.")][:1],
      [f"{subj.capitalize()} is hard.", f"The {subj} students are tired.", f"{subj.capitalize()} classes are full.", "The rules are clear.", "Sciences are fun.", "Her notes are good."], kind="Agreement", strategy=None)
