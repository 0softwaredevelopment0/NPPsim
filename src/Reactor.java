import java.util.ArrayList;
import java.util.Arrays;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;

/**
 * NPP-SIM — модель реактора ВВЭР-1000.
 * Точный порт проверенной JavaScript-модели (index.html), поведение идентично.
 */
public class Reactor {

    // ---- константы (единицы как в JS-версии) ----
    public static final double RATED = 3000;      // МВт(т)
    public static final double TAU = 1.5;         // постоянная времени мощности, с
    public static final double ROD_WORTH = 0.020; // полная отрицательная реактивность стержней
    public static final double ALPHA_T = 1.5e-5;  // температурный коэффициент, 1/°C
    public static final double KB = 1.0e-3;       // бор: реактивность на 1 г/кг от номинала 6 г/кг
    public static final double KHX = 100;         // МВт/°C съём тепла при полном расходе
    public static final double C1 = 15000;        // теплоёмкость 1 контура, МВт·с/°C
    public static final double C2 = 2200;         // теплоёмкость 2 контура, МВт·с/°C
    public static final double P1_SET = 15.7, P1_RELIEF = 18.6;
    public static final double T_WARN = 350, T_DAMAGE = 368;
    public static final double LI = 2.87e-5, LX = 2.1e-5, KI = 3.0e-6, KX = 7.0e-7, BURN = 2.0e-6;
    public static final double ROD_SLOW = 0.6, ROD_NORM = 2.5, ROD_FAST = 10;
    public static final double SCRAM_RATE = 45;
    public static final double EFF = 0.333;
    public static final double F0 = 50.0;
    public static final int ROD_N = 9;                 // число стержней ОР СУЗ (индивидуальное извлечение)
    public static final double MELT_T = 2800;          // температура плавления топлива, °C

    // ---- состояние ----
    public double P = 0.05, Tcore = 280, Tsg = 275, P1 = 15.7, P2 = 4.4;
    public double I = 4, pump = 1, feed = 0.5, gov = 0, B = 6.0;
    public double xe = 0, io = 0, level = 50, f = 50.0, MWe = 0;
    public double leakP = 0, pSet = 100, kdSlip = 0;   // kdSlip — накопленный слив давления при отключённой подпитке
    public int rodDir = 0;                 // -1/0/1
    public String rodSp = "norm";          // slow / norm / fast
    public int rate = 1;
    public boolean breaker = true, scram = false, loca = false, relP = false;
    public boolean relS = false, steamBreak = false, ark = true, dead = false;
    public boolean[] gcn = new boolean[4];      // ГЦН-1..4: ВКЛ/ОТКЛ
    public boolean[] fen = new boolean[3];      // ПЭН-1, ПЭН-2, МПНА
    public int bruMode = 1;                     // БРУ-А: 0 — ОТКЛ, 1 — АВТ, 2 — РУЧ
    public boolean bruManual = false;           // ручное открытие БРУ-А (режим РУЧ)
    public boolean bruWasOpen = false;          // гистерезис БРУ-А (открыта)
    public boolean makeup = true;               // подпитка компенсатора давления
    public int gpzCount = 0;                    // сколько раз открывались ГПЗ (сброс в атмосферу)
    public int azState = 0;            // АЗ: 0 — ВКЛ, 1 — СРАБОТАЛА (защёлкнута), 2 — ОТКЛЮЧЕНА оператором
    public double tScram = -1, dmgT = 0, dmgP = 0, dmgF = 0;
    public double time = 0;
    public double[] rodIns;              // ввод каждого стержня, % (0 — извлечён, 100 — полностью в зоне)
    public double[][] melt = new double[CORE_N][CORE_N];  // доля расплавленного топлива каждой ТК, 0..1

    // ---- журнал и сигнализация ----
    public static class Alarm {
        public final double t; public final String text; public final boolean warn; public boolean ack;
        public Alarm(double t, String text, boolean warn) { this.t = t; this.text = text; this.warn = warn; }
    }
    public final Map<String, Alarm> alarms = new LinkedHashMap<>();
    public static class LogEntry {
        public final double t; public final String msg; public final String cls;
        public LogEntry(double t, String msg, String cls) { this.t = t; this.msg = msg; this.cls = cls; }
    }
    public final List<LogEntry> log = new ArrayList<>();
    public boolean sound = true;

    // ---- задания (тренировка оператора) ----
    public static class Task {
        public final String text; public final int kind; public final double target; public final double hold;
        public double held = 0; public int state = 0;   // 0 — активно, 1 — выполнено, 2 — провал
        public Task(String text, int kind, double target, double hold) {
            this.text = text; this.kind = kind; this.target = target; this.hold = hold;
        }
    }
    public final List<Task> tasks = new ArrayList<>();
    private boolean allDone = false;

    // ---- тренды (точки: P%, Tcore, P1, MWe) ----
    public final List<double[]> trends = new ArrayList<>();
    private double sampT = 0;

    // ================= методы =================
    public static double clamp(double v, double a, double b) { return Math.max(a, Math.min(b, v)); }
    public static double rodWorthAt(double I) { return ROD_WORTH * Math.pow(I / 100, 1.2); }

    // ---- стержни ОР СУЗ: индивидуальные позиции (I — средний ввод) ----
    /** Установить ввод выбранных стержней, % (0 — извлечён, 100 — полностью в зоне). */
    public void setRods(int[] idx, double pct) {
        double v = clamp(pct, 0, 100);
        for (int k : idx) if (k >= 0 && k < ROD_N) rodIns[k] = v;
        recomputeI();
    }
    /** Установить ввод ВСЕХ стержней, %. */
    public void setAllRods(double pct) {
        double v = clamp(pct, 0, 100);
        for (int k = 0; k < ROD_N; k++) rodIns[k] = v;
        I = v;
    }
    /** Сдвинуть все стержни на dI (для ▲/▼, АРК, АЗ). */
    public void moveAllRods(double dI) {
        for (int k = 0; k < ROD_N; k++) rodIns[k] = clamp(rodIns[k] + dI, 0, 100);
        recomputeI();
    }
    private void recomputeI() {
        double s = 0; for (int k = 0; k < ROD_N; k++) s += rodIns[k];
        I = s / ROD_N;
    }
    /** Извлечение стержня k, % (100 — полностью извлечён из зоны). */
    public double rodExtract(int k) { return 100 - rodIns[k]; }
    /** Среднее извлечение всех стержней, %. */
    public double rodExtractMean() { return 100 - I; }
    /** Сколько ГЦН в работе. */
    public int gcnCount() { int n = 0; for (boolean b : gcn) if (b) n++; return n; }
    /** Расход 1-го контура (0..1) из числа работающих ГЦН. */
    public double pumpFlow() { return gcnCount() / 4.0; }
    /** Питательная вода (0..1): ПЭН-1 50%, ПЭН-2 50%, МПНА 30%. */
    public double feedFlow() {
        double f = (fen[0] ? 0.5 : 0) + (fen[1] ? 0.5 : 0) + (fen[2] ? 0.3 : 0);
        return Math.min(1.0, f);
    }
    /** БРУ-А открыт? АВТ — по давлению ПГ (с гистерезисом), РУЧ — по кнопке. */
    public boolean bruOpen() {
        if (bruMode == 2) return bruManual;
        if (bruMode != 1) return false;
        if (P2 >= 7.59) bruWasOpen = true;
        else if (P2 < 7.45) bruWasOpen = false;
        return bruWasOpen;
    }
    private double rodSpeed(String sp) {
        if ("slow".equals(sp)) return ROD_SLOW;
        if ("fast".equals(sp)) return ROD_FAST;
        return ROD_NORM;
    }
    private double satP2(double t) {
        double x = t - 280;
        return clamp(6.4 + 0.5 * x + 0.02 * x * x, 0.05, 7.6);
    }
    private double p2steamFrac() { return (gov / 0.9) * (P2 / 6.4); }

    public void physics(double dt) {
        int n = Math.max(1, (int) Math.ceil(dt / 0.5));
        double h = dt / n;
        for (int k = 0; k < n; k++) step(h);
    }

    private void step(double h) {
        // --- расход и питательная вода из состояния насосов (реальные переключатели) ---
        pump = pumpFlow();
        feed = feedFlow();
        // --- стержни (индивидуальные позиции, I — средний ввод) ---
        if (scram) moveAllRods(SCRAM_RATE * h);
        else if (rodDir != 0) moveAllRods(rodDir * rodSpeed(rodSp) * h);
        if (ark && !scram && rodDir == 0) {           // АРК: держит мощность на уставке
            double sp = rodSpeed("norm");
            if (P < pSet - 0.3) moveAllRods(-sp * h);
            else if (P > pSet + 0.3) moveAllRods(sp * h);
        }
        double rhoRod = -rodWorthAt(I);
        double rhoT = -ALPHA_T * (Tcore - 310);
        double rhoXe = -xe * 1.792e-4;                // 1.674 (равн.) => -3e-4
        double rhoB = -KB * (B - 6);
        double rho = rhoRod + rhoT + rhoXe + rhoB;

        // --- кинетика мощности ---
        double dP = P * rho / TAU * h;
        P = clamp(P + dP, 0.05, 250);
        if (scram && tScram >= 0) {
            double e = (time - tScram) / 2000;
            P = Math.max(P, 6.5 * Math.exp(-e) + 0.5);
        }
        double Pth = P / 100 * RATED;

        // --- ксенон/йод ---
        io += (KI * P - LI * io) * h;
        xe += (LI * io + KX * P - LX * xe - BURN * xe * P) * h;
        if (xe < 0) xe = 0;

        // --- 1 контур: температура ---
        double flowEff = pump + (pump < 0.02 ? 0.1 : 0);   // естественная циркуляция
        double leakF = loca ? 0.55 : 1;
        double Qrem = KHX * flowEff * leakF * (Tcore - Tsg);
        Tcore += (Pth - Qrem) / C1 * h;

        // --- давление 1 контура ---
        if (loca) leakP += 3.0 * h; else leakP = Math.max(0, leakP - 0.2 * h);
        if (makeup) kdSlip = Math.max(0, kdSlip - 0.002 * h);   // подпитка КД восстанавливает
        else kdSlip += 0.0015 * h;                              // подпитка ОТКЛ: медленный слив давления
        double p1 = 15.7 + 0.35 * (Tcore - 310) - leakP - kdSlip;
        if (relP) {
            p1 = Math.min(p1, 18.6); p1 -= 0.3 * h;
            if (p1 <= 18.6 && !(Tcore > 342)) relP = false;
        }
        P1 = clamp(p1, 0.1, 21);
        if (P1 > P1_RELIEF) { relP = true; alarm("P-1", "Сработал предохранительный клапан 1-го контура (P=" + f1(P1) + " МПа)"); }
        if (P1 > 18.0) protect("P1>18 МПа", "Срабатывание АЗ по давлению 1-го контура");
        if (P1 < 5.0)  protect("P1<5 МПа", "Срабатывание АЗ по низкому давлению 1-го контура");
        if (P1 < 0.8) { dmgP += h; } else dmgP = 0;

        // --- 2 контур ---
        double Qsteam = 3000 * p2steamFrac();
        if (steamBreak) Qsteam += 3500;
        boolean bruOpen = bruOpen();                       // по давлению на начало шага
        if (bruOpen) Qsteam += 3000;                       // БРУ-А: сброс пара отводит тепло
        if (relS) Qsteam += 3000;                          // ГПЗ: сброс пара в атмосферу отводит тепло
        Tsg += (Qrem - Qsteam) / C2 * h;
        P2 = satP2(Tsg);
        if (steamBreak) { P2 = Math.min(P2, 1.0); level = Math.max(0, level - 1.5 * h); }
        // ГПЗ срабатывают только если БРУ-А не сбрасывает пар (по фактическому P2)
        boolean bruNow = (bruMode == 1 && P2 >= 7.59) || (bruMode == 2 && bruManual);
        if (P2 >= 7.6 && !bruNow && !relS) { relS = true; gpzCount++; alarm("S-1", "Сработали ГПЗ (сброс пара в атмосферу)"); }
        if (relS && P2 < 7.4) relS = false;
        if (bruNow && !alarms.containsKey("BRU")) alarm("BRU", "БРУ-А открыт: сброс пара в конденсатор");

        // --- уровень ПГ ---
        double sf = clamp(p2steamFrac(), 0, 1.5);
        level = clamp(level + (feed - sf) * 3 * h, 0, 120);
        if (level < 15) {
            if (gov > 0) tripTurbine("Защита ПГ: низкий уровень");
            if (P > 30) protect("уровень ПГ<15%", "АЗ по низкому уровню ПГ");
        }
        if (level < 10) protect("уровень ПГ<10%", "АЗ по низкому уровню ПГ");
        if (P2 < 1.2 && gov > 0) tripTurbine("Падение давления пара");
        if (level < 25 || level > 85) alarm("L-1", "Уровень ПГ за пределами: " + Math.round(level) + "%");

        // --- турбина и генератор ---
        double MWeAvail = Pth * EFF;
        MWe = (gov > 0 && breaker) ? Math.max(0, Math.min(MWeAvail, Qsteam * EFF)) : 0;
        f = F0 + (breaker ? clamp((MWe - 1000) / 30000, -0.3, 0.3) : 0);

        // --- защиты ---
        if (P1 > P1_RELIEF) alarm("P-2", "Предельное давление 1-го контура!");
        if (P > 115) protect("P>115%", "Срабатывание АЗ по превышению мощности");
        if (Tcore > T_WARN) {
            alarm("T-1", "Превышение температуры активной зоны: " + Math.round(Tcore) + "°C");
            if (Tcore > 350) protect("Tcore>350°C", "АЗ по температуре активной зоны");
        }
        if (Tcore > T_DAMAGE) { dmgT += h; } else dmgT = 0;
        if (flowEff < 0.25 && Tcore > 350) { dmgF += h; } else dmgF = 0;
        if (flowEff < 0.25 && P > 30) protect("потеря расхода", "АЗ по потере расхода 1-го контура");
        int nGcn = gcnCount();
        if (nGcn <= 2 && P > 10) protect("отключение 2 ГЦН", "АЗ по отключению двух ГЦН");
        if (nGcn <= 3 && P > 75) protect("отключение ГЦН на мощности", "АЗ: отключение ГЦН при мощности > 75%");

        // --- повреждение ---
        if ((dmgT > 30 || dmgP > 30 || dmgF > 90) && !dead) {
            dead = true; scram = true; gov = 0; MWe = 0;
            String cause = dmgT > 30 ? "расплавление активной зоны из-за перегрева"
                    : (dmgP > 30 ? "обнажение активной зоны из-за падения давления"
                    : "перегрев активной зоны при потере расхода");
            addLog("КРИТ: повреждение активной зоны — " + cause, "crit");
            alarm("R-1", "РАДИАЦИОННАЯ АВАРИЯ! Повреждение активной зоны");
        }

        // --- температура топлива и расплавление ТК ---
        double fmax = 0;
        for (int i = 0; i < CORE_N; i++) for (int j = 0; j < CORE_N; j++) {
            double tf = fuelTemp(i, j);
            if (tf > fmax) fmax = tf;
            if (tf > MELT_T) {
                melt[i][j] = Math.min(1, melt[i][j] + (tf - MELT_T) / MELT_T * h / 60);
                if (melt[i][j] >= 1 && !dead) {
                    dead = true; scram = true; gov = 0; MWe = 0;
                    addLog("КРИТ: РАСПЛАВЛЕНИЕ ТОПЛИВА в ТК (" + (i + 1) + "," + (j + 1) + ") — темп. "
                            + Math.round(tf) + "°C > " + Math.round(MELT_T) + "°C", "crit");
                    alarm("R-2", "РАДИАЦИОННАЯ АВАРИЯ: расплавление топлива активной зоны");
                }
            }
        }
        if (fmax > 2600) alarm("T-3", "Предельная температура топлива: " + Math.round(fmax) + "°C");
    }

    public void doScram(String msg) {
        if (scram) return;
        scram = true; tScram = time; gov = 0; azState = 1;   // АЗ сработала и ВЫКЛЮЧИЛАСЬ — защёлкнута
        addLog("АВАРИЙНАЯ ЗАЩИТА: " + msg, "crit");
        addLog("АЗ выключена после срабатывания — взведите её перед перезапуском", "warn");
        alarm("AZ", "Срабатывание АЗ — ввод стержней");
        beep();
    }
    /** Автозащита: срабатывает только при ВКЛЮЧЁННОЙ АЗ. */
    private void protect(String cond, String msg) {
        if (azState == 0) { doScram(msg); return; }
        if (azState == 2) {
            if (!alarms.containsKey("AZ-OFF")) {          // логируем однократно, не спамим журнал
                addLog("!! АЗ ОТКЛЮЧЕНА — " + msg + " НЕ сработала!", "crit");
                alarm("AZ-OFF", "АЗ отключена: защита не сработала (" + cond + ")");
            }
        }
    }
    /** Оператор: включить/отключить АЗ (отключение автозащиты — опасно!). */
    public void toggleAz() {
        if (azState == 1) { addLog("АЗ сработала и защёлкнута — сначала взведите её", "warn"); return; }
        azState = (azState == 0) ? 2 : 0;
        if (azState == 2) { addLog("!! АЗ ОТКЛЮЧЕНА оператором — автозащита НЕ сработает", "crit"); alarm("AZ-OFF", "АЗ отключена оператором"); }
        else addLog("АЗ включена. Автозащита активна.", "ok");
    }
    public void tripTurbine(String msg) {
        if (gov == 0) return;
        gov = 0;
        addLog("ОСТАНОВ ТУРБИНЫ: " + msg, "warn");
        alarm("T-2", "Останов турбины: " + msg);
        beep();
    }
    public void scram() { doScram("Нажатие кнопки АЗ оператором"); }
    public void resetScram() {
        if (scram) { scram = false; azState = 0; addLog("АЗ взведена (сброс). Разрешено управление стержнями.", "ok"); return; }
        if (azState == 2) { azState = 0; addLog("АЗ включена. Автозащита активна.", "ok"); }
    }
    public void closeBreak() {
        if (!loca && !steamBreak) return;
        loca = false; steamBreak = false;
        addLog("Течь устранена / паропровод изолирован.", "ok");
    }
    public void ackAll() { for (Alarm a : alarms.values()) a.ack = true; }

    // ================= сценарии =================
    public void loadScenario(String k) {
        if ("full".equals(k)) {
            resetState(); P = 100; Tcore = 310; Tsg = 280; P1 = 15.7; P2 = 6.4;
            setAllRods(3); gov = 0.9; breaker = true; B = 5.4;
            Arrays.fill(gcn, true); Arrays.fill(fen, false); fen[0] = true; fen[1] = true;
            xe = 1.674; io = 10.45; level = 50; rate = 1; ark = true; pSet = 100;
            addLog("=== НОВАЯ СМЕНА / СЦЕНАРИЙ: РАБОТА 100% ===", "ok");
            addLog("Задание: удерживайте параметры в норме, используя все средства управления.", "info");
            tasks.add(new Task("Удерживайте мощность 90–110% (2 мин)", 0, 90, 120));
            tasks.add(new Task("Давление 1-го контура 15–18 МПа (2 мин)", 9, 0, 120));
            tasks.add(new Task("Не допускайте срабатывания АЗ", 10, 0, 0));
            tasks.add(new Task("Не допускайте повреждения активной зоны", 11, 0, 0));
        } else if ("start".equals(k)) {
            resetState(); P = 0.05; Tcore = 280; Tsg = 275; P1 = 15.7; P2 = 4.4;
            setAllRods(4); gov = 0; breaker = true; B = 6.0;
            Arrays.fill(gcn, true); Arrays.fill(fen, false); fen[0] = true;
            xe = 0; io = 0; level = 50; rate = 60; ark = false; pSet = 50;
            addLog("=== НОВАЯ СМЕНА / СЦЕНАРИЙ: ПУСК БЛОКА ===", "ok");
            addLog("Задание: управляйте стержнями, бором, ГЦН, турбиной — выведите блок на мощность.", "info");
            tasks.add(new Task("Поднимите мощность выше 50% (30 с)", 0, 50, 30));
            tasks.add(new Task("Включите турбину — впуск пара > 0", 2, 0, 0));
            tasks.add(new Task("Выйдите на эл. мощность ≥ 950 МВт (30 с)", 3, 950, 30));
            tasks.add(new Task("Не допускайте срабатывания АЗ", 10, 0, 0));
            tasks.add(new Task("Не допускайте повреждения активной зоны", 11, 0, 0));
        } else if ("stop".equals(k)) {
            resetState(); P = 100; Tcore = 310; Tsg = 280; P1 = 15.7; P2 = 6.4;
            setAllRods(3); gov = 0.9; breaker = true; B = 5.4;
            Arrays.fill(gcn, true); Arrays.fill(fen, false); fen[0] = true; fen[1] = true;
            xe = 1.674; io = 10.45; level = 50; rate = 10; ark = true; pSet = 20;
            addLog("=== НОВАЯ СМЕНА / СЦЕНАРИЙ: ОСТАНОВ ===", "ok");
            addLog("Задание: остановите блок, используя АРК, стержни и управление турбиной.", "info");
            tasks.add(new Task("Снизьте мощность ниже 10% (1 мин)", 1, 10, 60));
            tasks.add(new Task("Введите стержни ОР СУЗ в зону (≥ 95%)", 4, 95, 0));
            tasks.add(new Task("Не допускайте срабатывания АЗ", 10, 0, 0));
            tasks.add(new Task("Не допускайте повреждения активной зоны", 11, 0, 0));
        }
    }

    private void resetState() {
        P = 0.05; Tcore = 280; Tsg = 275; P1 = 15.7; P2 = 4.4;
        I = 4; rodDir = 0; rodSp = "norm"; pump = 1; feed = 0.5; gov = 0;
        breaker = true; B = 6.0; xe = 0; io = 0;
        Arrays.fill(gcn, true);
        Arrays.fill(fen, false); fen[0] = true; fen[1] = true;
        bruMode = 1; bruManual = false; bruWasOpen = false; makeup = true; gpzCount = 0;
        rodIns = new double[ROD_N]; Arrays.fill(rodIns, 4);
        melt = new double[CORE_N][CORE_N];
        scram = false; tScram = -1; ark = true; pSet = 100;
        steamBreak = false; loca = false; relP = false; relS = false;
        level = 50; f = 50.0; MWe = 0; dmgT = 0; dmgP = 0; dmgF = 0;
        leakP = 0; kdSlip = 0; dead = false; sampT = 0; azState = 0;
        alarms.clear(); log.clear(); trends.clear(); tasks.clear(); allDone = false;
    }

    // ================= события (аварии) =================
    public void fireEvent(String k) {
        String ev = k;
        if ("random".equals(k)) {
            String[] pool = {"pump", "loca", "steam", "scram", "feed", "grid", "rod"};
            ev = pool[(int) (Math.random() * pool.length)];
        }
        if (dead) { addLog("Блок в аварийном состоянии — загрузите сценарий", "warn"); return; }
        addLog("!!! ВОЗНИКЛО СОБЫТИЕ: " + ev.toUpperCase(), "warn");
        switch (ev) {
            case "pump":
                for (int i = 0; i < 4; i++) if (gcn[i]) { gcn[i] = false; addLog("Отказ ГЦН-" + (i + 1) + " — расход снизился", "crit"); break; }
                alarm("EV", "Отказ ГЦН"); addTask(new Task("Включите все ГЦН (4 из 4)", 5, 0.8, 20)); break;
            case "loca": loca = true; addLog("Обнаружена течь 1-го контура! Давление падает", "crit"); alarm("EV", "Течь 1-го контура"); addTask(new Task("Устраните течь 1-го контура", 6, 0, 0)); break;
            case "steam": steamBreak = true; addLog("Разрыв паропровода! Давление 2-го контура падает", "crit"); alarm("EV", "Разрыв паропровода"); addTask(new Task("Изолируйте разорванный паропровод", 6, 0, 0)); break;
            case "scram": doScram("Ложное срабатывание АЗ (событие)"); break;
            case "feed":
                for (int i = 0; i < 3; i++) if (fen[i]) { fen[i] = false; addLog("Отказ " + (i == 2 ? "МПНА" : "ПЭН-" + (i + 1)) + " — питательная вода снизилась", "crit"); break; }
                alarm("EV", "Отказ питательного насоса"); addTask(new Task("Восстановите питательную воду ≥ 80%", 8, 0.8, 20)); break;
            case "grid": if (breaker) { breaker = false; addLog("Отключение от сети — генератор отпал", "warn"); addTask(new Task("Подключите генератор к сети", 7, 0, 0)); } break;
            case "rod": int rk = (int) (Math.random() * ROD_N); rodIns[rk] = clamp(rodIns[rk] + 30, 0, 100); recomputeI();
                addLog("Падение кассеты стержня " + (rk + 1) + " — ввод " + Math.round(rodIns[rk]) + "%, реактивность снизилась", "warn"); alarm("EV", "Падение кассеты"); break;
        }
    }

    // ================= служебные =================
    public void addLog(String msg, String cls) {
        log.add(new LogEntry(time, msg, cls == null ? "info" : cls));
        if (log.size() > 400) log.remove(0);
    }
    public void alarm(String key, String text) { alarm(key, text, false); }
    public void alarm(String key, String text, boolean warn) {
        if (alarms.containsKey(key)) return;
        alarms.put(key, new Alarm(time, text, warn));
        beep();
    }
    private void beep() {
        if (sound) java.awt.Toolkit.getDefaultToolkit().beep();
    }
    /** Прогресс времени: вызвать каждый тик с шагом dt (уже поделённым на rate). */
    public void advance(double dt) {
        if (!dead) physics(dt);
        time += dt;
        sampT += dt;
        if (sampT >= 1) {
            sampT = 0;
            trends.add(new double[]{P, Tcore, P1, MWe});
            if (trends.size() > 1500) trends.remove(0);
        }
        updateTasks(dt);
    }

    // ================= задания (тренировка) =================
    public void addTask(Task t) {
        for (Task x : tasks) if (x.text.equals(t.text)) return;
        tasks.add(t);
        allDone = false;   // новая цель после завершения смены — снова отслеживаем
    }
    private void updateTasks(double dt) {
        if (tasks.isEmpty() || allDone) return;
        boolean othersDone = true;
        for (Task t : tasks) if (t.state == 0 && !isFailKind(t.kind)) { othersDone = false; break; }
        for (Task t : tasks) {
            if (t.state != 0) continue;
            if (t.kind == 10 && scram) { t.state = 2; addLog("✘ ПРОВАЛ ЗАДАНИЯ: " + t.text, "crit"); continue; }
            if (t.kind == 11 && dead) { t.state = 2; addLog("✘ ПРОВАЛ ЗАДАНИЯ: " + t.text, "crit"); continue; }
            boolean ok = checkTask(t);
            if (ok) {
                t.held += dt;
                if (isFailKind(t.kind)) { if (othersDone) { t.state = 1; addLog("✔ Задание выполнено: " + t.text, "ok"); } }
                else if (t.held >= t.hold) { t.state = 1; addLog("✔ Задание выполнено: " + t.text, "ok"); }
            } else t.held = 0;
        }
        for (Task t : tasks) if (t.state != 1) return;
        allDone = true;
        addLog("=== СМЕНА УСПЕШНО ЗАВЕРШЕНА — все задания выполнены ===", "ok");
    }
    private boolean checkTask(Task t) {
        switch (t.kind) {
            case 0: return P >= t.target;
            case 1: return P <= t.target;
            case 2: return gov > 0;
            case 3: return MWe >= t.target;
            case 4: return I >= t.target;
            case 5: return pump >= t.target;
            case 6: return !loca && !steamBreak;
            case 7: return breaker;
            case 8: return feed >= t.target;
            case 9: return P1 >= 15 && P1 <= 18;
            case 10: return !scram;   // fail-kind
            case 11: return !dead;    // fail-kind
            default: return false;
        }
    }
    private boolean isFailKind(int k) { return k == 10 || k == 11; }

    // ================= активная зона: температура ТК =================
    public static final int CORE_N = 9;
    /** Температура кассеты (ТК) i,j: центр горячее краёв, с шумом. */
    public double faTemp(int i, int j) {
        double c = (CORE_N - 1) / 2.0;
        double d = Math.hypot(i - c, j - c) / c;                       // 0 — центр, 1 — край
        double hot = 40 * clamp(P / 100, 0, 1) * Math.max(0, 1 - d * d * 1.15);
        double noise = (((i * 7 + j * 13) % 5) - 2) * 1.4;
        return Math.max(120, Tcore - 8 + hot + noise);
    }
    // ================= температура топлива и расплавление =================
    /** Температура ТОПЛИВА ТК i,j: теплоноситель + подогрев от мощности и теплосъёма. */
    public double fuelTemp(int i, int j) {
        double flowEff = pump + (pump < 0.02 ? 0.1 : 0);
        double leakF = loca ? 0.55 : 1;
        double cool = clamp(0.35 + 0.65 * flowEff * leakF, 0.35, 1.6);
        double c = (CORE_N - 1) / 2.0;
        double d = Math.hypot(i - c, j - c) / c;
        double radial = Math.max(0, 1 - d * d * 1.15);
        double heat = 1400 * Math.pow(clamp(P / 100, 0, 2.5), 1.5) / cool;
        return faTemp(i, j) + heat * radial;
    }
    /** Максимальная температура топлива по всем ТК, °C. */
    public double fuelMax() {
        double m = 0;
        for (int i = 0; i < CORE_N; i++) for (int j = 0; j < CORE_N; j++) m = Math.max(m, fuelTemp(i, j));
        return m;
    }
    /** Сколько ТК расплавлено полностью. */
    public int meltedCount() {
        int c = 0;
        for (int i = 0; i < CORE_N; i++) for (int j = 0; j < CORE_N; j++) if (melt[i][j] >= 1) c++;
        return c;
    }
    /** Суммарная доля расплавленного топлива (0..N). */
    public double meltTotal() {
        double s = 0;
        for (int i = 0; i < CORE_N; i++) for (int j = 0; j < CORE_N; j++) s += melt[i][j];
        return s;
    }
    public static String f1(double v) { return String.format("%.1f", v); }
    public static String f2(double v) { return String.format("%.2f", v); }
}
