const fs = require('fs');
const html = fs.readFileSync('C:/NPPsim/index.html', 'utf8');
const script = html.match(/<script>([\s\S]*?)<\/script>/)[1];

const cache = {};
function el() {
  return {
    style: {}, classList: { toggle() {}, add() {}, remove() {} },
    setAttribute() {}, getAttribute() { return null; },
    appendChild() {}, removeChild() {},
    children: [], firstChild: null, scrollTop: 0, scrollHeight: 0,
    textContent: '', innerHTML: '', className: '', value: '0',
    clientWidth: 800, clientHeight: 400, width: 0, height: 0,
    getContext: () => ({ setTransform(){}, clearRect(){}, beginPath(){}, moveTo(){}, lineTo(){}, stroke(){}, fill(){}, fillText(){}, strokeStyle:'', lineWidth:1, fillStyle:'' }),
    _a: 0
  };
}
const document = {
  getElementById: id => cache[id] || (cache[id] = el()),
  createElement: () => el(),
  querySelectorAll: () => []
};
const window = { addEventListener() {}, devicePixelRatio: 1 };
const requestAnimationFrame = () => {};
const performance = { now: () => Date.now() };

const testCode = `
const fmt = () => 'P='+S.P.toFixed(1)+'% T='+S.Tcore.toFixed(1)+' P1='+S.P1.toFixed(2)+' P2='+S.P2.toFixed(2)+' MWe='+S.MWe.toFixed(0)+' xe='+S.xe.toFixed(4)+' dmgT='+S.dmgT.toFixed(0)+' dmgP='+S.dmgP.toFixed(0)+' dead='+S.dead;
const run = (sec) => { for (let i = 0; i < sec * 10; i++) { physics(0.1); S.time += 0.1; } };
let fails = 0;
const check = (name, ok, extra) => { console.log((ok ? 'PASS' : 'FAIL') + ' ' + name + (extra ? '  [' + extra + ']' : '')); if (!ok) fails++; };

loadScenario('full');
run(7200);
console.log('steady@100% after 2h:', fmt());
check('power stays near 100%', Math.abs(S.P - 100) < 15);
check('Tcore sane', S.Tcore > 300 && S.Tcore < 345);
check('P1 sane', S.P1 > 15.2 && S.P1 < 18.0);
check('P2 sane', S.P2 > 5.5 && S.P2 < 7.6);
check('MWe sane', S.MWe > 850 && S.MWe < 1020);

const xeBefore = S.xe;
scram();
run(900);
console.log('after scram 15min:', fmt());
check('power dropped after scram', S.P < 12);
check('xenon pit appears (xe rises)', S.xe > xeBefore * 1.05);
check('no damage on clean scram', !S.dead);

run(3600 * 20);
console.log('20h after scram:', fmt());
check('xenon decaying after pit', S.xe < 4.0);

loadScenario('full');
S.loca = true;
run(300);
console.log('LOCA after 5min:', fmt());
check('LOCA drops P1', S.P1 < 1);
check('LOCA leads to damage', S.dead);

loadScenario('full');
S.pump = 0;
run(900);
console.log('pumps off 15min:', fmt());
check('pump loss triggers auto scram', S.scram === true);
check('pump loss does NOT immediately damage', !S.dead);

loadScenario('full');
S.feed = 0;
run(1800);
console.log('feed loss 30min:', fmt());
check('feed loss auto scram', S.scram === true);
check('feed loss recoverable (no dead)', !S.dead);

loadScenario('full');
S.ark = false; S.B = 4.0; S.I = 0;
run(1500);
console.log('reactivity excursion 25min:', fmt());
check('reactivity excursion triggers auto AZ', S.scram === true);
check('reactivity excursion does NOT damage (AZ protects)', !S.dead);

loadScenario('start');
S.B = 5.5; S.I = 0;
run(3600);
console.log('startup rods out + dilute 1h:', fmt());
check('startup power rose', S.P > 1);
check('startup no damage', !S.dead);

loadScenario('full');
S.steamBreak = true;
run(20);
console.log('steam break 20s:', fmt());
check('P2 collapsed', S.P2 < 5.5);
S.feed = 1; closeBreak();
run(1200);
console.log('after closing break 20min:', fmt());
check('system recovers after closing break', S.P > 50 && S.P2 > 5.5 && !S.dead);

console.log(fails === 0 ? '\\nALL TESTS PASSED' : '\\n' + fails + ' TEST(S) FAILED');
process.exit(fails === 0 ? 0 : 1);
`;

try {
  new Function('document', 'window', 'performance', 'requestAnimationFrame',
    '"use strict";' + script + '\n' + testCode)(
    document, window, performance, requestAnimationFrame
  );
} catch (e) {
  console.log('RUNTIME ERROR:', e.message, e.stack.split('\n')[1]);
  process.exit(1);
}
