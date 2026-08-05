/**
 * Сквозной прогон физики модели (порт проверок из _test.js).
 * Запуск: java PhysicsTest
 */
public class PhysicsTest {

    static int fails = 0;

    static void check(String name, boolean ok, String extra) {
        System.out.println((ok ? "PASS " : "FAIL ") + name + (extra == null ? "" : "  [" + extra + "]"));
        if (!ok) fails++;
    }

    static String fmt(Reactor S) {
        return String.format("P=%.1f%% T=%.1f P1=%.2f P2=%.2f MWe=%.0f xe=%.4f dmgT=%.0f dmgP=%.0f dead=%b",
                S.P, S.Tcore, S.P1, S.P2, S.MWe, S.xe, S.dmgT, S.dmgP, S.dead);
    }

    public static void main(String[] args) {
        Reactor S = new Reactor();
        java.util.function.Consumer<Double> run = sec -> {
            double t = sec * 10;
            for (int i = 0; i < (int) t; i++) { S.physics(0.1); S.time += 0.1; }
        };

        // 1. Устойчивость на 100% в течение 2 ч
        S.loadScenario("full");
        run.accept(7200.0);
        System.out.println("steady@100% after 2h: " + fmt(S));
        check("power stays near 100%", Math.abs(S.P - 100) < 15, null);
        check("Tcore sane", S.Tcore > 300 && S.Tcore < 345, null);
        check("P1 sane", S.P1 > 15.2 && S.P1 < 18.0, null);
        check("P2 sane", S.P2 > 5.5 && S.P2 < 7.6, null);
        check("MWe sane", S.MWe > 850 && S.MWe < 1020, null);

        // 2. АЗ: спад мощности, йодная яма, без повреждений
        double xeBefore = S.xe;
        S.scram();
        run.accept(900.0);
        System.out.println("after scram 15min: " + fmt(S));
        check("power dropped after scram", S.P < 12, null);
        check("xenon pit appears (xe rises)", S.xe > xeBefore * 1.05, null);
        check("no damage on clean scram", !S.dead, null);

        // 3. Спад ксенона после ямы
        run.accept(3600.0 * 20);
        System.out.println("20h after scram: " + fmt(S));
        check("xenon decaying after pit", S.xe < 4.0, null);

        // 4. LOCA: падение давления, повреждение
        S.loadScenario("full");
        S.loca = true;
        run.accept(300.0);
        System.out.println("LOCA after 5min: " + fmt(S));
        check("LOCA drops P1", S.P1 < 1, null);
        check("LOCA leads to damage", S.dead, null);

        // 5. Потеря расхода: автоматическое АЗ, без мгновенного повреждения
        S.loadScenario("full");
        S.gcn = new boolean[4];                 // все ГЦН отключены
        run.accept(900.0);
        System.out.println("pumps off 15min: " + fmt(S));
        check("pump loss triggers auto scram", S.scram, null);
        check("pump loss does NOT immediately damage", !S.dead, null);

        // 6. Отказ питательной воды: АЗ, авария переживаема
        S.loadScenario("full");
        S.fen = new boolean[3];                 // все питательные насосы отключены
        run.accept(1800.0);
        System.out.println("feed loss 30min: " + fmt(S));
        check("feed loss auto scram", S.scram, null);
        check("feed loss recoverable (no dead)", !S.dead, null);

        // 7. Разгон реактивности: АЗ защищает
        S.loadScenario("full");
        S.ark = false; S.B = 4.0; S.setAllRods(0);
        run.accept(1500.0);
        System.out.println("reactivity excursion 25min: " + fmt(S));
        check("reactivity excursion triggers auto AZ", S.scram, null);
        check("reactivity excursion does NOT damage (AZ protects)", !S.dead, null);

        // 8. Пуск: извлечение стержней + разбавление бора поднимает мощность
        S.loadScenario("start");
        S.B = 5.5; S.setAllRods(0);
        run.accept(3600.0);
        System.out.println("startup rods out + dilute 1h: " + fmt(S));
        check("startup power rose", S.P > 1, null);
        check("startup no damage", !S.dead, null);

        // 9. Разрыв паропровода: восстановление после устранения
        S.loadScenario("full");
        S.steamBreak = true;
        run.accept(20.0);
        System.out.println("steam break 20s: " + fmt(S));
        check("P2 collapsed", S.P2 < 5.5, null);
        S.fen = new boolean[]{true, true, false}; S.closeBreak();   // ПЭН-1 + ПЭН-2
        run.accept(1200.0);
        System.out.println("after closing break 20min: " + fmt(S));
        check("system recovers after closing break", S.P > 50 && S.P2 > 5.5 && !S.dead, null);

        // 10. АЗ: сработала → защёлкнута (выключена) до взвода
        S.loadScenario("full");
        check("AZ armed initially", S.azState == 0, null);
        S.scram();
        check("AZ latched after trip", S.azState == 1 && S.scram, null);
        S.resetScram();
        check("AZ re-armed after взвод", S.azState == 0 && !S.scram, null);

        // 11. Отключённая АЗ не срабатывает автоматически, ручная кнопка работает
        S.loadScenario("full");
        S.toggleAz();
        check("AZ disabled by operator", S.azState == 2, null);
        S.gcn = new boolean[]{true, true, false, false};   // 2 ГЦН (50% расхода, автозащита НЕ срабатывает)
        run.accept(120.0);
        check("disabled AZ does NOT auto-trip", !S.scram, null);
        check("no melt without extreme conditions", S.meltedCount() == 0 && !S.dead, null);
        S.scram();                  // ручная кнопка действует всегда
        check("manual scram works when AZ disabled", S.scram && S.azState == 1, null);

        // 12. Задания: движок засчитывает цели оператора
        S.loadScenario("start");
        S.P = 60; S.gov = 0.9; S.fen = new boolean[]{true, true, false};
        S.gcn = new boolean[]{true, true, true, true}; S.setAllRods(2); S.B = 5.5;
        for (int i = 0; i < 400; i++) S.advance(0.1);   // 40 с симуляции
        boolean pDone = false, govDone = false;
        for (Reactor.Task t : S.tasks) {
            if (t.text.startsWith("Поднимите") && t.state == 1) pDone = true;
            if (t.text.contains("турбину") && t.state == 1) govDone = true;
        }
        check("task: power goal completed", pDone, null);
        check("task: turbine goal completed", govDone, null);

        System.out.println(fails == 0 ? "\nALL TESTS PASSED" : "\n" + fails + " TEST(S) FAILED");
        // 13. Индивидуальное извлечение стержней (% извлечения)
        S.loadScenario("full");
        S.setRods(new int[]{0, 4}, 100);                        // два стержня полностью в зоне
        check("per-rod set changes mean insertion", Math.abs(S.I - (7 * 3 + 2 * 100) / 9.0) < 1e-9, null);
        S.setAllRods(0);                                        // «выбрать все» → извлечь полностью
        check("select-all extraction -> I=0", S.I == 0, null);
        check("extraction readback 100%", Math.abs(S.rodExtractMean() - 100) < 1e-9, null);
        S.setRods(new int[]{2}, 100);                           // один стержень полностью в зону
        check("single rod inserted -> mean 100/9", Math.abs(S.I - 100.0 / 9) < 1e-9, null);

        // 14. На 100% мощности топливо НЕ плавится (температура под контролем)
        S.loadScenario("full");
        run.accept(7200.0);
        System.out.println("fuel after 2h: max=" + Reactor.f1(S.fuelMax()) + "°C melted=" + S.meltedCount());
        check("fuel temp controlled at rated power", S.fuelMax() < 2800 && S.meltedCount() == 0, null);

        // 15. Расплавление: АЗ отключена + разгон → плавление топлива, авария
        S.loadScenario("full");
        S.toggleAz(); S.ark = false; S.B = 3.5; S.setAllRods(0); S.P = 220;
        run.accept(150.0);
        System.out.println("excursion melt: max=" + Reactor.f1(S.fuelMax()) + " melted=" + S.meltedCount() + " dead=" + S.dead);
        check("fuel melts on uncontrolled excursion", S.meltedCount() >= 1 && S.dead, null);

        // 16. АЗ по превышению мощности защищает от расплавления
        S.loadScenario("full");
        S.ark = false; S.B = 4.0; S.setAllRods(0);
        run.accept(1500.0);
        System.out.println("excursion AZ on: P=" + Reactor.f1(S.P) + " scram=" + S.scram + " dead=" + S.dead + " melted=" + S.meltedCount());
        check("power-trip AZ prevents melt", S.scram && !S.dead && S.meltedCount() == 0, null);

        // 17. ГЦН: индивидуальное включение, расход от числа насосов, АЗ при отключении двух
        S.loadScenario("full");
        check("4 ГЦН → расход 100%", Math.abs(S.pumpFlow() - 1.0) < 1e-9, null);
        check("все ПЭН вкл → пит. вода 100%", Math.abs(S.feedFlow() - 1.0) < 1e-9, null);
        S.gcn[3] = false;
        check("3 ГЦН → расход 75%", Math.abs(S.pumpFlow() - 0.75) < 1e-9, null);
        S.gcn[2] = false; S.gcn[1] = false;
        run.accept(30.0);
        System.out.println("2 GCN off: " + fmt(S));
        check("отключение 2 ГЦН на мощности → АЗ", S.scram, null);

        // 18. БРУ-А: АВТ держит давление ПГ, ОТКЛ ведёт к ГПЗ (которые сбрасывают пар и переживаемы)
        S.loadScenario("full");
        S.gov = 0.3; S.bruMode = 0;             // турбина сбросила нагрузку, БРУ выключена
        run.accept(600.0);
        System.out.println("BRU OFF: P2=" + Reactor.f1(S.P2) + " gpz=" + S.gpzCount + " dead=" + S.dead);
        check("БРУ ОТКЛ → ГПЗ перехватили сброс", S.gpzCount >= 1 && !S.dead, null);
        S.loadScenario("full");
        S.gov = 0.3; S.bruMode = 1;             // БРУ в АВТ
        run.accept(600.0);
        System.out.println("BRU AUTO: P2=" + Reactor.f1(S.P2) + " gpz=" + S.gpzCount + " bruOpen=" + S.bruOpen());
        check("БРУ АВТ держит давление (без ГПЗ)", S.gpzCount == 0 && S.P2 > 6.0, null);
        // РУЧ: с кнопкой сбрасывает так же, как АВТ
        S.loadScenario("full");
        S.gov = 0.3; S.bruMode = 2; S.bruManual = true;
        run.accept(600.0);
        check("БРУ РУЧ+открыта держит давление", S.gpzCount == 0 && S.P2 > 6.0, null);

        // 19. Подпитка КД: отключена → давление медленно падает, блок в порядке
        S.loadScenario("full");
        S.makeup = false;
        run.accept(3600.0);
        System.out.println("no makeup 1h: P1=" + Reactor.f1(S.P1));
        check("отключённая подпитка снижает P1", S.P1 < 15.5 && !S.dead, null);

        System.exit(fails == 0 ? 0 : 1);
    }
}
