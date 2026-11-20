"""Reviewed skill mechanics shared by the native mod and browser preview.

The browser preview is the approval surface for changes to these facts.
Descriptions use true raw, native element/status units, and 30-fps frame timings.
"""
GUIDE='https://docs.google.com/document/d/1W0FDr-Ynzj9qFcEwXhbPNdGrN-1Pj20MAYKUuofEUiE/edit'
JP='https://www.kuroyonhon.com/mhxx/program/skill.php'
# Binary effects do not have a meaningful percentage or duration to invent.
BASE={
 'No Skill':'No skill effect is active.',
 'Negate Poison':'Blocks Poison and Noxious Poison. Deadly Poison is downgraded to ordinary Poison; it is not complete Deadly Poison immunity.',
 'Double Poison':'Poison health-drain damage x2 while poisoned.',
 'Negate Paralysis':'Prevents the hunter from being paralyzed; always active while equipped.',
 'Double Paralysis':'Paralysis duration x2 whenever paralysis is inflicted.',
 'Negate Sleep':'Prevents sleep status; always active while equipped.',
 'Double Sleep':'Sleep duration x2 whenever sleep is inflicted.',
 'Negate Stun':'Prevents stun completely; always active while equipped.',
 'Halve Stun':'50% chance to cancel a stun when it would trigger. Does not halve stun duration or simply double the buildup threshold.',
 'Double Stun':'Stun recovery takes twice as many recovery inputs.',
 'HG Earplugs':'Ignores small and large roar flinches. Does not prevent damage or other effects carried by a roar.',
 'Earplugs':'Ignores small-roar flinches; large roars still interrupt you.',
 'Windproof (Hi)':'Blocks small and large wind pressure. Dragon wind is reduced to large wind, not fully negated.',
 'Windproof (Lo)':'Blocks small wind-pressure staggers; large and dragon wind still affect you.',
 'Tremor Res':'Prevents ground-tremor stagger from monster attacks; does not negate the hit damage.',
 'Negate Bind':'Prevents Snowman, Boned/Ossified and severe Bubbleblight movement restriction. Does not prevent Mucus.',
 'Heat Cancel':'Prevents hot-area health drain and lava-floor damage; no Cool Drink required. Does not grant Fireblight immunity.',
 'Heat Surge':'Hot-area and lava-floor health-drain damage x2.',
 'Cold Cancel':'Prevents cold-area maximum-stamina loss; no Hot Drink required.',
 'Cold Surge':'Cold-caused maximum-stamina loss is twice as fast.',
 'Polar Hunter':'In cold areas: +15 raw attack and +20 Defense; ignores cold. An active Cool Drink adds +5 raw attack in any area.',
 'Tropic Hunter':'In hot areas: +15 raw attack and +20 Defense; ignores heat/lava drain. An active Hot Drink adds +10 Defense in any area.',
 'Anti-Theft':'Prevents monsters from stealing items from your pouch.',
 'Iron Skin':'Prevents both normal and severe Defense Down.',
 'Antivirus':'On overcoming Frenzy: affinity bonus rises from +15% to +30%. Frenzy is easier to overcome; no benefit before infection/recovery.',
 'Bio Researcher':'Prevents Stench and Blastblight; Dung Bomb combines succeed 100%. A dung-susceptible monster attempts to leave when hit, if it can change areas.',
 'Bio Master':'Bio Researcher effects: Stench/Blastblight immunity and guaranteed Dung Bomb combines/flee attempt. Also delays Frenzy infection; exact infection-rate modifier is not verified.',
 'Negate Bleeding':'Prevents Bleeding; always active while equipped.',
 'Double Bleeding':'Health loss caused by Bleeding x2.',
 'Blightproof':'Prevents Fire, Water, Thunder, Ice and Dragonblight, including severe elemental blights. Does not prevent Blastblight.',
 'Element Atk Up':'Weapon elemental value x1.10. With an element-specific Atk skill, the combined multiplier is capped at x1.20; flat additions are separate.',
 'Element Atk Down':'Weapon elemental value x0.90; only applies to elements already present on the weapon.',
 'Status Atk +1':'Poison/Paralysis/Sleep buildup value x1.10, then +1. Does not multiply the damage of poison ticks or affect Blast.',
 'Status Atk +2':'Poison/Paralysis/Sleep buildup value x1.20, then +1. Does not multiply poison-tick damage or affect Blast.',
 'Status Atk Down':'Poison/Paralysis/Sleep buildup value x0.90; does not affect Blast.',
 'Speed Sharpening':'Whetstone sharpening uses 1 stroke instead of 4 (75% fewer strokes). Does not increase sharpness restored.',
 'Slow Sharpening':'Whetstone sharpening uses 5 strokes instead of 4.',
 'Sharpness +1':'Adds up to 25 sharpness units, only into the weapon\'s extendable sharpness segment. No effect on a non-extendable gauge.',
 'Sharpness +2':'Adds up to 50 sharpness units, only into the weapon\'s extendable sharpness segment. No effect on a non-extendable gauge.',
 'Razor Sharp':'1-unit sharpness costs have a 50% chance to cost 0. Costs of 2+ units are halved, rounded down.',
 'Blunt Edge':'Sharpness consumption x2 on attacks that consume sharpness.',
 "Mind's Eye":'Prevents sharpness/hitzone-based bouncing. Does not increase damage or remove the extra sharpness cost of a would-be bounce.',
 'Blind Eye':'33% chance for a melee hit to bounce even if its sharpness/hitzone would normally allow the hit.',
 'Heavy Polish':'After sharpening, lasts 60 seconds: melee sharpness consumption becomes 0; gunner critical-distance multiplier becomes x1.75 instead of x1.50. Item Use Up/Down modifies the duration.',
 'Bludgeoner':'Raw attack multiplier by current sharpness: green x1.10, yellow x1.15, orange/red x1.20; blue or better gets no melee bonus.',
 'Critical Draw':'+100 percentage points affinity on weapon draw attacks; capped at 100% final affinity. Hunter Arts do not count as draw attacks.',
 'Punishing Draw':'+5 raw attack. Cutting draw attacks also apply 30 KO and 20 exhaust; Dual Blades use different per-hit values, which are not verified.',
 'Quick Sheath':'Sheathing animation speed x1.30 (about 23% less animation time). Actual full-action time depends on weapon/style; also speeds up Valor sheath-cancel preparation.',
 'Challenge Sheath':'35% chance when sheathing with an enraged large monster in the area to restore sharpness: GS 4; LS 10; SnS/DB/Lance 15; Hammer/HH 11; GL/CB 16; SA 14; IG 12. Automatic LS roundslash sheathing does not trigger it.',
 'Bladescale Hone':'While drawn: 5 rolls or 9 hops restore 5 sharpness (7 with Seregios melee). Bowguns: 50% per evade to load 1 selected shot; Seregios stacking gives 1 guaranteed plus 50% for a second. Bow close-range coating: raw x1.50 (x1.65 with Seregios).',
 'Normal/Rapid Up':'Normal S and Rapid arrows: raw damage x1.10. Does not increase elemental damage.',
 'Pierce/Pierce Up':'Pierce S and Pierce arrows: raw damage x1.10. Does not increase elemental damage.',
 'Pellet/Spread Up':'Pellet S raw damage x1.20; Spread-arrow raw damage x1.30.',
 'Heavy/Heavy Up':'Heavy S and Heavy-arrow damage x1.10.',
 'Bonus Shot':'Adds 1 shot to each Light Bowgun rapid-fire volley. Does not increase individual-shot damage.',
 'Shot Booster':'Extends critical-distance range by 50%; does not increase the critical-distance damage multiplier.',
 'Load Up':'Bowguns/Gunlance: +1 shot per clip. Bow: unlocks charge level 4 if normally locked. Charge Blade: yellow charge fills 5 phials instead of 3; red fills up to 10 instead of 5.',
 'TrueShot Up':'Bowgun internal-ammo damage x1.20. Bow Power/Heavy shots: raw x1.20; Arc Shots: raw x1.40. Bow elemental damage is unaffected.',
 'Ammo Saver':'20% chance per shot/coating use to avoid consuming it. Bowgun ammo is returned to reserve, not automatically reloaded.',
 'Weakness Exploit':'+50 percentage points affinity when the struck part has a physical hitzone of at least 45 for your damage type.',
 'Repeat Offender':'After 1 hit: +25% affinity; after 5 consecutive hits: +30% total, not +55%. Each hit refreshes the 5-second timer; 5 seconds without a hit removes the bonus.',
 'Status Crit':'On critical hits, Poison/Paralysis/Sleep buildup x1.20. Does not increase Blast or poison-tick damage.',
 'Elemental Crit':'On critical hits, element multiplier: SnS/DB/Bow x1.35; LBG/HBG x1.30; GS x1.20; other weapons x1.25.',
 'Critical Boost':'Positive raw critical hits deal x1.40 damage instead of x1.25. Does not raise affinity or change negative critical hits.',
 'Bitter Affinity':'When a negative-affinity hit occurs, 25% chance to replace its x0.75 raw multiplier with x2.00. This is not a normal critical hit for Critical Boost/Elemental Crit/Status Crit.',
 'Focus':'GS/Hammer/Bow charge time x0.80; LS Spirit, DB Demon, SA and CB gauge gain x1.25. LBG hunter-art gain is also accelerated; its exact modifier is not verified.',
 'Distraction':'GS/Hammer/Bow charge time x1.20; LS Spirit, DB Demon, SA and CB gauge gain x0.75. LBG hunter-art charging is slowed.',
 'Marathon Runner':'Continuous stamina drain (sprinting/Demon Mode/charging) x0.50. Does not reduce the fixed cost of an evade or block.',
 'Short Sprinter':'Continuous stamina drain x1.20 (+20%); fixed evade/block costs and movement speed are unchanged.',
 'Constitution +1':'Fixed stamina costs such as evades and blocks x0.75 (25% less). Does not change continuous sprint drain.',
 'Constitution +2':'Fixed stamina costs such as evades and blocks x0.50 (50% less). Does not change continuous sprint drain.',
 'Constitution -1':'Fixed stamina costs such as evades and blocks x1.20.',
 'Constitution -2':'Fixed stamina costs such as evades and blocks x1.35.',
 'Stam Recov Up':'Stamina regeneration rate x2.00 while stamina is recovering.',
 'Stam Recov Down':'Stamina regeneration rate x0.50 while stamina is recovering.',
 'Endurance Runner':'While sprinting at 25 stamina or less, continuous sprint drain stops. Fixed stamina costs can still consume stamina.',
 'Evasion +1':'Normal evade invulnerability: 10 frames instead of 6 at 30 fps (0.333 s instead of 0.200 s). Does not stack with the equivalent small-bubble evasion effect.',
 'Evasion +2':'Normal evade invulnerability: 12 frames instead of 6 at 30 fps (0.400 s instead of 0.200 s).',
 'Evasion Down':'Normal evade invulnerability: 3 frames instead of 6 at 30 fps (0.100 s instead of 0.200 s).',
 'Evade Extender':'Normal evade travel distance x1.50. Does not lengthen invulnerability or extend Hunter Art evades.',
 'Bubbly Dance':'Prevents severe Bubbleblight. After 3 rolls or 6 hops, small bubbles grant Evasion +1 and 25% lower fixed stamina costs. Stamina reduction stacks with Constitution; being hit/interrupted pops the bubbles.',
 'Guard +1':'+10 guard strength. Crossing a guard threshold reduces recoil, chip damage and stamina cost; it is not a flat damage reduction.',
 'Guard +2':'+20 guard strength. Crossing a guard threshold reduces recoil, chip damage and stamina cost; it is not a flat damage reduction.',
 'Guard -1':'-10 guard strength; can cause a stronger recoil/chip/stamina-cost category for the same blocked attack.',
 'Guard Up':'Allows guarding against attacks designated blockable only with Guard Up (including many beams/gases). Does not block grabs or every normally unblockable attack.',
 'Knockout King':'KO buildup x1.10 for attacks that already deal KO; does not give ordinary cutting hits KO. Kinsect KO is excluded.',
 'Stamina Thief':'Exhaust buildup x1.20 for attacks that already deal exhaust. Does not increase HP damage or KO buildup.',
 'Horn Maestro':'Extends song-specific Hunting Horn timers: e.g. Attack Up (L) 90 -> 120 seconds. Also reduces horn/flute breakage and healing-song downgrades; exact item probabilities are not verified.',
 'Artillery Novice':'Cannon/Ballista x1.10; Crag/Triburst/Wyvern/Cannon S raw x1.15; CB impact explosions x1.30; GL fixed shell damage x1.10. Red GL heat changes are reduced by one third.',
 'Artillery Expert':'Cannon/Ballista x1.20; Crag/Triburst/Wyvern/Cannon S raw x1.30; CB impact explosions x1.35; GL fixed shell damage x1.20. Red GL heat changes are halved; Wyvern\'s Fire cooldown 120 -> 90 seconds.',
 'Partbreaker':'Break/sever buildup x1.30 on eligible unbroken parts. Does not increase HP damage; the part must still meet its damage-type/break conditions.',
 'Bombardier':'Bomb-item damage x1.30, Blast buildup x1.20; bomb-combination success becomes 100%. Does not amplify Blast explosion damage.',
 'Latent Power +1':'+30% affinity and 50% less stamina consumption for 90 seconds. Triggers after 180 damage taken or 300 seconds fighting a large monster.',
 'Latent Power +2':'+50% affinity and 75% less stamina consumption for 90 seconds. Triggers after 180 damage taken or 300 seconds fighting a large monster.',
 'Challenger +1':'+10 raw attack and +10% affinity while a large monster in your area is enraged.',
 'Challenger +2':'+20 raw attack and +15% affinity while a large monster in your area is enraged.',
 'Peak Performance':'+20 raw attack only while current HP equals maximum HP; red recoverable HP does not count as full health.',
 'Trump Card':'A large-monster knockdown/immobilization grants a 60-second buff: damaging Hunter Arts generally x1.20 raw and timed Arts x1.20 duration. Art-specific overrides are listed in Full effects.',
 'Dragonheart':'At HP <= 2/3 maximum: raw attack x1.10, all elemental resistances become 50, and special Dragonblight suppresses weapon elements/statuses except Dragon. Also prevents ordinary Dragonblight.',
 'Adrenaline +1':'At HP <= 40%: +15 Defense above the normal low-HP Defense bonus; no attack increase.',
 'Adrenaline +2':'At HP <= 40%: raw attack x1.30 and +15 Defense above the normal low-HP Defense bonus.',
 'Worrywart':'At HP <= 40%: raw attack x0.70; the normal low-HP Defense bonus is reduced by 9.',
 'Fortify':'Each time YOU faint: +10% attack and +15% Defense, up to 2 stacks (+20% attack/+30% Defense). Teammate carts do not trigger it.',
 'Resentment':'+20 raw attack while any red recoverable HP remains on your health bar.',
 'Resuscitate':'+20 raw attack while afflicted by a qualifying negative status or elemental blight; small Bubbleblight from Bubbly Dance also qualifies.',
 'Guts':'Once per life, survives an otherwise lethal hit at 1 HP if you had at least 65 HP before the hit. Resets after fainting; damage over time or a second hit can still kill you.',
 'Sneak':'Lowers aggro gained from actions, so monsters are less likely to select you. Targeting also depends on other players/actions and monster AI; no universal target-chance percentage is verified.',
 'Taunt':'Raises aggro gained from actions. It is not guaranteed targeting; exact aggro gain and final target odds are not verified.',
 'Palico Rally':'Accompanying Palicoes: attack and Defense x1.10 (rounded down). MHGU does not gain the older Shakalaka stamina-recovery effect.',
 'Palico Cheer':'Use the Fist Pump gesture to recall fainted/recovering Palicoes. Reuse cooldown: 600 seconds.',
 'Mounting Master':'Mount buildup and mounted-topple gauge gain x1.25. Does not increase ordinary aerial HP damage.',
 'Saddle Sore':'Mount buildup and mounted-topple gauge gain x0.90.',
 'Airborne':'Eligible jumping attacks: raw damage, mount buildup and aerial-topple buildup x1.10. Combined mount multipliers are capped at x1.40. Grounded attacks are unchanged.',
 'Composed':'Hunter Art, Valor and Alchemy gauge gain x1.15.',
 'Chance Seizer':'Taking monster damage grants Hunter Art/Valor/Alchemy gauge. Gain is damage-dependent; exact conversion coefficients are not verified.',
 'Longer SP':'SP-state duration x1.50. This affects SP support duration, not the Hunter Art\'s own animation.',
 'Autotracker':'Always marks large monsters on the map, with species/facing and awareness information; no Paintball required.',
 'Detect':'Marked/revealed large monsters show species, facing and awareness color: blue unaware, yellow aware, red fighting. Still needs Paintball or another reveal source.',
 'Capture Guru':'A marked/revealed monster\'s map icon flashes white when its species-specific capture HP threshold is reached. Does not change that threshold or guarantee trap success.',
 'Outdoorsman':'Map is always visible without carrying the item. Fish-bite and BBQ timing windows are extended by 50%.',
 'Pro Transporter':'Heavy delivery items are carried at light-item speed: about +33% movement speed. Light items do not gain this speed bonus. Also allows higher drop-offs; the safe drop height is not verified.',
 'Divine Blessing':'25% chance on a damage event to reduce damage by 30% (x0.70).',
 'Demonic Blessing':'25% chance on a damage event to increase damage by 30% (x1.30).',
 "Hero's Talisman":'Negates an attack\'s damage and associated reactions/statuses when its post-Defense damage is below 5 HP.',
 'Recovery Up':'Healing amount from eligible recovery effects x1.25. Does not accelerate passive recovery of the red HP bar.',
 'Recovery Down':'Healing amount from eligible recovery effects x0.75.',
 'Recovery Spd +1':'Passive recovery of red HP x2.00; ordinary green-HP healing is unchanged.',
 'Recovery Spd +2':'Passive recovery of red HP x4.00; ordinary green-HP healing is unchanged.',
 'Recovery Spd -1':'Passive recovery of red HP x0.50.',
 'Recovery Spd -2':'Passive recovery of red HP x0.25.',
 'Item Use Up':'Eligible timed consumable buffs last x1.50 as long; e.g. Heavy Polish 60 -> 90 seconds. Does not affect every timed item or Hunter Art.',
 'Item Use Down':'Eligible timed consumable buffs last x2/3 as long; e.g. Heavy Polish 60 -> 40 seconds.',
 'Wide-Range +1':'Eligible consumed items also affect hunters in the same area at 50% strength; cures/seeds have item-specific handling. Includes Potion, Herb, Antidote, Nulberry and Might/Adamant Seed.',
 'Wide-Range +2':'Eligible consumed items also affect hunters in the same area at 100% strength. Includes Potion, Herb, Antidote, Nulberry and Might/Adamant Seed; not Mega Potion or every consumable.',
 'Negate Hunger':'Maximum-stamina loss from time passing becomes 0. Does not prevent stamina spent by actions.',
 'Halve Hunger':'Time-based maximum-stamina loss rate x0.50; ordinary 25-point loss interval doubles from 6 to 12 minutes.',
 'Raise Hunger':'Time-based maximum-stamina loss interval shortens from 6 to 4.5 minutes per 25 points.',
 'Double Hunger':'Time-based maximum-stamina loss interval shortens from 6 to 3 minutes per 25 points.',
 'Scavenger':"Meat receives Gourmand's extra 25 maximum stamina. Other eligible food/drink consumables have a 1/3 chance to restore 25 maximum stamina; capped at 150.",
 'Gourmand':'Eligible meat restores an extra 25 maximum stamina, guaranteed when the meat successfully restores stamina. Well-done Steak restores 75 instead of 50; maximum stamina remains capped at 150.',
 'Speed Eating +1':'Uses the fast eating animation for meat only. Drinking Potion/other non-meat items is unchanged; no universal full-animation percentage is verified.',
 'Speed Eating +2':'Uses fast eating/drinking animations for oral consumables, including meat and Potions. The finishing flex remains; exact total time depends on item/animation.',
 'Slow Eater':'Uses the slow meat-eating animation. Other item types are unchanged; exact item-specific timings are not verified.',
 'Rationer':'25% chance when eating/drinking an eligible item to keep that item instead of consuming it.',
 'Meat Lover':'Raw Meat becomes edible and restores 50 maximum stamina. Cooked meat other than Burnt Meat also grants 60 seconds of unlimited stamina, timed from starting to eat (roughly 55 seconds after the animation).',
 'Mushromancer':'Eat Blue Mushroom for Potion, Dragon Toadstool for Max Potion, Nitroshroom for Demondrug, and Mopeshroom for Dash Juice. Full effects lists other edible mushrooms.',
 'Herbology':'Herbs restore 30 HP, before other recovery modifiers.',
 'Pro Herbology':'Herbs restore 50 HP, before other recovery modifiers.',
 'Combination Pro':'Successful combinations that have variable output always produce their maximum quantity; does not increase combination success chance.',
 'Trap Master':'Trap combines succeed 100%; trap/bomb placement speed x2.00.',
 'Gathering +1':'Gathering continuation roll rises from 25/32 to 27/32 (78.125% -> 84.375%). Does not change which item is rolled.',
 'Gathering +2':'Gathering continuation roll rises from 25/32 to 29/32 (78.125% -> 90.625%). Does not change which item is rolled.',
 'Gathering -1':'Gathering continuation roll falls from 25/32 to 21/32 (78.125% -> 65.625%).',
 'Honey Hunter':'Each Honey or Royal Honey gather yields 2 items instead of 1.',
 'Charm Collector':'When mining a charm, 20% chance to get one additional charm.',
 'Charm Chaser':'When mining a charm, 30% chance to get one additional charm.',
 'Carving Pro':'Prevents small knockbacks while carving. Does not add carves or protect against every damaging hit.',
 'Carving Celebrity':'+1 carve per eligible body/tail and protection from small carving knockbacks. Does not change carve drop-table odds.',
 'Capture Expert':'Capture reward count is at least 3 instead of 2-3. Does not improve individual item drop probabilities.',
 'Capture Master':'At least 3 capture rewards, plus a 2/3 chance for a fourth reward. Does not improve individual item drop probabilities.',
 'Torso Up':'Copies chest armor skill points, including chest jewels, once per equipped Torso Up piece. N copies give chest points x(N+1); does not copy Defense or resistances.',
 'Skill +2':'+2 points to each OTHER skill tree with a nonzero positive total. Does not add points to absent trees or to itself.',
 'Double Talisman':'Talisman skill points x2, including decorations fitted in the talisman. Does not double armor/weapon jewels.',
}

FULL={
 'Mushromancer':'Edible mushroom mapping: Blue Mushroom -> Potion; Toadstool -> Nutrients; Dragon Toadstool -> Max Potion; Parashroom/Bindshroom -> Armorskin; Nitroshroom -> Demondrug; Super Mushroom -> Mega Potion; Mopeshroom -> Dash Juice; Choice Mushroom -> Mega Dash Juice; Abyssal Mushroom -> Nulberry; Reststool -> Antidote; Chaos Mushroom -> Psychoserum; Unique Mushroom -> Ration; Ripened Mushroom -> Ancient Potion. Exact mappings for Great Shroom and other rare event mushrooms are not verified.',
 'Trump Card':'Trigger: a large monster is knocked down or immobilized; buff lasts 60 seconds. Most damaging Arts: raw x1.20; most timed Arts: duration x1.20. Special cases: Absolute Evasion/Readiness perform 2 evades; Frenzy Fever grants its affinity effect immediately; Lion\'s Maw next-hit multiplier gains 15 percentage points; Euphony gets second-level melodies; Wyvern\'s Breath fire/fixed damage +20%; Demon Riot effect +15%; Full House uses faster animation; Gunpowder Infusion and Charged Shot load 5 extra shots; Devouring Demon damage +10% and duration +20%. Arisen Phoenix heals more HP, but its exact extra amount is not verified.',
}

def build(native):
 import re
 effects=dict(BASE)
 for i,tier in enumerate(('S','M','L')):
  effects[f'Attack Up ({tier})']=f'+{(10,15,20)[i]} flat raw attack; always active while equipped, before percentage-based attack modifiers.'
  effects[f'Attack Down ({tier})']=f'-{(5,10,15)[i]} flat raw attack; always active while equipped.'
  effects[f'Defense Up ({tier})']=(
   '+15 flat Defense.', 'Defense x1.03, then +20 flat Defense.', 'Defense x1.06, then +25 flat Defense.')[i]
  effects[f'Defense Down ({tier})']=(
   '-10 flat Defense.', 'Defense x0.95, then -10 flat Defense.', 'Defense x0.90, then -10 flat Defense.')[i]
 for element in ('Fire','Water','Thunder','Ice','Dragon'):
  for level,percent,flat in [(1,5,4),(2,10,6)]:
   effects[f'{element} Atk +{level}']=f'{element} value x{1+percent/100:.2f}, then +{flat} {element.lower()}; only if the weapon already has that element. With Element Atk Up, percentage multipliers cap at x1.20.'
  effects[f'{element} Atk Down']=f'{element} value x0.75 on weapons/ammo carrying that element; no effect on other elements.'
  for value in (20,15,-20):effects[f'{element} Res {value:+}']=f'{value:+} {element.lower()} resistance, active while equipped. Each resistance point changes incoming elemental damage by 1 percentage point; it does not alter raw damage.'
 for health in (50,20,-10,-30):effects[f'Health {health:+}']=f'{health:+} maximum HP, subject to the 150-HP cap. Does not stack past that cap with food/items.'
 for level,affinity in [(1,10),(2,20),(3,30),(-1,-5),(-2,-10),(-3,-15)]:
  effects[f'Critical Eye {level:+}']=f'{affinity:+} percentage points affinity, active while equipped. Final normal affinity is capped at +100%.'
 for level in (1,2,3,-1,-2,-3):
  n=abs(level);effects[f'Reload Speed {level:+}']=f'Bowgun reload rating {level:+} stage'+('s'if n>1 else'')+'. Actual reload time depends on gun and ammo thresholds. '+('Bow coatings load automatically.'if level>=2 else'Bow coating application is faster.'if level==1 else'Bow coating application is slower.')
  effects[f'Recoil Down {level:+}']=f'Bowgun recoil rating {-level:+} stage'+('s'if n>1 else'')+'. Actual recoil animation changes only when the ammo/gun crosses a recoil threshold.'
 for level in (1,2,-1,-2):effects[f'Steadiness {level:+}']=f'Bowgun deviation {-level:+} stage'+('s'if abs(level)>1 else'')+'. +2 removes native deviation; it does not increase shot damage.'
 for kind in ('Normal','Pierce','Pellet','Crag','Clust'):
  for any_level in (False,True):
   name=f'Use Any {kind} S'if any_level else f'Use Lv1 {kind} S'
   if name in native:effects[name]=f'Enables {kind} S '+('levels 1-3'if any_level else'level 1')+' on a bowgun that lacks them. Clip size depends on the gun; it does not grant free ammo or a damage bonus.'
 for coating in ('Poison','Para','Sleep','C.Range','Exhaust','Blast'):
  effects[f'Use {coating} Coat']=f'Enables {coating} coatings on a bow that cannot normally use them. Does not raise coating potency on a bow that already supports them.'
 for kind in ('Power','Element'):
  for tier in ('Lv1','All'):
   effects[f'{kind} Phial ({tier})']=f'Enables {kind.lower()} bow coatings '+('levels 1 and 2'if tier=='All'else'level 1')+' if the bow lacks them; this is coating access, not a Switch Axe/Charge Blade phial bonus.'
 for chance in (45,20,-10,-20):effects[f'Combination {chance:+}%']=f'{chance:+} percentage points combination success chance; final chance is capped at 100%.'
 for name,roll in [('Miraculous Luck',31),('Great Luck',28),('Good Luck',25),('Bad Luck',16),('Horrible Luck',8)]:
  effects[name]=f'After guaranteed quest-reward slots, continuation roll becomes {roll}/32 ({roll/32*100:.3f}%) instead of 22/32 (68.750%), until the reward-row cap. Does not alter individual item drop rates.'
 for name,mult in [("Spirit's Whim",2/3),('Divine Whim',.5),("Spectre's Whim",4/3),("Devil's Whim",1.5)]:
  effects[name]=f'Pickaxe/Bug Net/Boomerang/flute/horn break chance x{mult:.3f} per use; multiplies the tool\'s own break chance, not an absolute proc percentage.'
 # Compound identities come from the native game's text, not a third-party
 # table: e.g. Iron Wall uses Defense M, Pack Rat uses Gathering +1.
 components={}
 for name,description in native.items():
  if 'Combines the effects of' in description:
   components[name]=[v.strip(', .')for v in re.findall(r'"([^\"]+)"',description)]
 def resolve(name,trail=()):
  if name in effects:return effects[name]
  if name not in components:raise KeyError('No explicit mechanic description: '+name)
  if name in trail:raise ValueError('Compound cycle: '+name)
  effects[name]=' '.join(child+': '+resolve(child,trail+(name,))for child in components[name])+' Shared component effects do not stack with the same separately activated skill.'
  return effects[name]
 for name in native:resolve(name)
 return {name:effects[name] for name in native},components

SOURCE_OVERRIDES={
 'Gourmand':['https://wikiwiki.jp/nenaiko/スキル/食いしん坊','https://game-e.com/mhxx/skill/111.php'],
 'Scavenger':['https://wikiwiki.jp/nenaiko/スキル/食いしん坊','https://game-e.com/mhxx/skill/111.php'],
 'Meat Lover':['https://wikiwiki.jp/nenaiko/スキル/肉食'],
 'Quick Sheath':['https://note.com/name_tengudake/n/nfc95bb7c3b39'],
 'Pro Transporter':['https://wikiwiki.jp/nenaiko/スキル/運搬'],
 'Palico Rally':['https://wikiwiki.jp/nenaiko/スキル/采配'],
 'Short Sprinter':['https://wikiwiki.jp/nenaiko/スキル/スタミナ'],
 'Saddle Sore':['https://wikiwiki.jp/nenaiko/スキル/乗り'],
 'Herbology':[JP,'https://jp.samurai-gamers.com/mhxx/skill-y-1/'],
 'Horn Maestro':['https://docs.google.com/document/d/1eG5dLgWW_QoRAu4DVLg3-PfcwQw1pjWeun6gqAvoaIU/edit','https://www.reddit.com/r/MHGU/comments/wg0586/what_does_the_hh_finale_melody_do/'],
}


def descriptions(native):
 """Return compact captions and complete explanations by native activation name."""
 import re
 effects,components=build(native)
 def full(name):
  if name in components:
   return '\n'.join(child+': '+full(child)for child in components[name])+'\nShared skill components do not stack with separate copies of themselves.'
  return FULL.get(name,effects[name])
 result={}
 for name,text in effects.items():
  complete=full(name)
  short=text
  if len(short)>245:
   short='; '.join(child+': '+effects[child].split('. ')[0]for child in components[name]) if name in components else text
   if len(short)>245:short=short[:230].rsplit(' ',1)[0]+'…'
  if re.search(r'not verified|not establish|no universal.*verified',complete) and 'not verified' not in short:
   short+=' Some secondary values are unverified; see Full effect.'
  result[name]=(short,complete)
 return result


def pack(texts):
 """Byte-pair dictionary for immutable captions, decoded one skill at a time.

 Unused byte values are tokens, so UTF-8 literals remain lossless. Each new
 pair refers only to older tokens; expansion is acyclic by construction.
 """
 from collections import Counter
 raw=[list(t.encode('utf-8'))for t in dict.fromkeys(texts)]
 used={v for row in raw for v in row}
 pairs=[(0,0)]*256
 for token in (i for i in range(256)if i not in used):
  counts=Counter(p for row in raw for p in zip(row,row[1:]))
  counts.pop((0,0),None)  # Reserved literal sentinel in the native decoder.
  if not counts:break
  pair,count=counts.most_common(1)[0]
  if count<3:break
  pairs[token]=pair
  for index,row in enumerate(raw):
   out=[];i=0
   while i<len(row):
    if i+1<len(row)and(row[i],row[i+1])==pair:out.append(token);i+=2
    else:out.append(row[i]);i+=1
   raw[index]=out
 blob=bytearray();locations={}
 for text,row in zip(dict.fromkeys(texts),raw):
  if len(text.encode())>4096:raise ValueError('Skill effect exceeds decode buffer')
  locations[text]=(len(blob),len(row));blob.extend(row)
 return blob,pairs,[locations[t]for t in texts]
