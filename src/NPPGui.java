import javax.swing.*;
import javax.swing.border.LineBorder;
import javax.swing.text.*;
import java.awt.*;
import java.awt.event.*;
import java.awt.geom.AffineTransform;
import java.util.ArrayList;
import java.util.Arrays;
import java.util.List;

/** NPP-SIM — настольный интерфейс (Swing). Тёмная тема «диспетчерская». */
public class NPPGui {

    private final Reactor R = new Reactor();

    // ---- палитра ----
    static final Color BG = new Color(0x0a0e12), PANEL = new Color(0x10161d), PANEL2 = new Color(0x151d26);
    static final Color LINE = new Color(0x22303d), TXT = new Color(0xc8d6e0), DIM = new Color(0x5f7385);
    static final Color GREEN = new Color(0x3ddc84), AMBER = new Color(0xffb454), RED = new Color(0xff5a5a);
    static final Color CYAN = new Color(0x4cc9f0), YELLOW = new Color(0xffe14d);

    private JLabel clock, hdrMWe, lamp, statusText;
    private JLabel[] bigVal = new JLabel[6];
    private JLabel rodPos, feedVal, govVal, borVal, pSetVal, freqVal, arkLbl;
    private JLabel azStatus, gcnLbl, bruLamp;
    private JSlider govSl, borSl, pSetSl;
    private JToggleButton sndBtn;
    private JButton resetAzBtn, azToggleBtn, bruOpenBtn, az1Btn, az2Btn;
    private JComboBox<String> bruModeCb;
    private ToggleSwitch[] gcnSw = new ToggleSwitch[4];
    private ToggleSwitch[] fenSw = new ToggleSwitch[3];
    private ToggleSwitch makeupSw, brkSw;
    private JButton rsSlow, rsNorm, rsFast;
    private DefaultListModel<Reactor.Alarm> alarmModel = new DefaultListModel<>();
    private JTextPane logPane;
    private SchemePanel scheme;
    private CorePanel core;
    private TrendPanel trends;
    private JList<Reactor.Alarm> alarmList;
    private JPanel tasksBox;
    private String taskSig = "";
    private boolean overlayShown = false;
    private boolean[] rodSel = new boolean[Reactor.ROD_N];
    private JTextField rodPctField;
    private JLabel rodMeanLbl, rodSelLbl;

    private List<String> logKeys = new ArrayList<>();

    public NPPGui() {
        JFrame f = new JFrame("NPP-SIM — Симулятор управления АЭС (ВВЭР-1000)");
        try {
            java.net.URL iconUrl = NPPGui.class.getResource("/app-icon.png");
            if (iconUrl != null) f.setIconImage(new javax.swing.ImageIcon(iconUrl).getImage());
        } catch (Exception ignored) { }
        f.setDefaultCloseOperation(JFrame.EXIT_ON_CLOSE);
        f.setSize(1360, 860);
        f.setLocationRelativeTo(null);
        f.setBackground(BG);

        JPanel content = new JPanel(new BorderLayout(0, 0));
        content.setBackground(BG);

        content.add(buildHeader(), BorderLayout.NORTH);

        JSplitPane split = new JSplitPane(JSplitPane.HORIZONTAL_SPLIT);
        split.setBorder(null);
        split.setDividerSize(4);
        split.setResizeWeight(0.75);
        split.setLeftComponent(buildLeft());
        split.setRightComponent(buildRight());
        split.setDividerLocation(1000);
        content.add(split, BorderLayout.CENTER);

        bindKeys(content);

        f.setContentPane(content);
        f.setVisible(true);

        R.loadScenario("full");
        R.addLog("Диспетчерская: симулятор запущен. Клавиши: ↑/↓ — стержни, A — АЗ, пробел — стоп", "ok");

        Timer t = new Timer(100, e -> tick());
        t.start();
    }

    // ================= ВЕРХНЯЯ ПАНЕЛЬ =================
    private JPanel buildHeader() {
        JPanel h = new JPanel(new BorderLayout());
        h.setBackground(PANEL);
        h.setBorder(BorderFactory.createCompoundBorder(
                BorderFactory.createMatteBorder(0, 0, 1, 0, LINE),
                BorderFactory.createEmptyBorder(6, 10, 6, 10)));

        JLabel title = new JLabel("<html><b style='font-size:15px;color:#4cc9f0'>NPP-SIM</b> &nbsp;<span style='font-size:11px;color:#5f7385'>СИМУЛЯТОР УПРАВЛЕНИЯ АЭС • ВВЭР-1000</span></html>");
        h.add(title, BorderLayout.WEST);

        JPanel right = new JPanel(new FlowLayout(FlowLayout.RIGHT, 8, 0));
        right.setOpaque(false);
        clock = lbl("00:00:00", 14, CYAN, true);
        right.add(kv("Время", clock));
        hdrMWe = lbl("1000", 15, GREEN, true);
        right.add(kv("МВт(э)", hdrMWe));
        right.add(rateBtn("1×", 1)); right.add(rateBtn("10×", 10));
        right.add(rateBtn("60×", 60)); right.add(rateBtn("600×", 600));
        sndBtn = new JToggleButton("🔊 Звук");
        styleBtn(sndBtn); sndBtn.setSelected(true);
        sndBtn.addActionListener(e -> R.sound = sndBtn.isSelected());
        right.add(sndBtn);
        lamp = lbl("●", 14, GREEN, false);
        statusText = lbl("РАБОТА", 13, GREEN, true);
        JPanel st = new JPanel(new FlowLayout(FlowLayout.RIGHT, 6, 0));
        st.setOpaque(false);
        st.add(lamp); st.add(statusText);
        st.setBorder(new LineBorder(LINE, 1));
        st.setBackground(PANEL2);
        st.setOpaque(true);
        right.add(st);
        h.add(right, BorderLayout.EAST);
        return h;
    }

    private JPanel kv(String k, JLabel v) {
        JPanel p = new JPanel(new BorderLayout());
        p.setOpaque(false);
        JLabel kl = new JLabel(k);
        kl.setFont(kl.getFont().deriveFont(9f));
        kl.setForeground(DIM);
        p.add(kl, BorderLayout.NORTH);
        p.add(v, BorderLayout.CENTER);
        p.setBorder(BorderFactory.createEmptyBorder(0, 6, 0, 6));
        return p;
    }

    private JButton rateBtn(String txt, int r) {
        JButton b = new JButton(txt);
        styleBtn(b);
        b.addActionListener(e -> { R.rate = r; });
        return b;
    }

    // ================= ЛЕВАЯ КОЛОНКА: схема + индикаторы + графики =================
    private JPanel buildLeft() {
        JPanel left = new JPanel(new BorderLayout());
        left.setBackground(BG);

        scheme = new SchemePanel();
        core = new CorePanel();
        JPanel cards = new JPanel(new CardLayout());
        cards.add(core, "core");
        cards.add(scheme, "scheme");
        CardLayout cl = (CardLayout) cards.getLayout();

        JToggleButton tbCore = new JToggleButton("Обзор реактора");
        JToggleButton tbSch = new JToggleButton("Схема блока");
        styleBtn(tbCore); styleBtn(tbSch);
        tbCore.setSelected(true);
        tbCore.setBackground(new Color(0x1a3a2a));
        ButtonGroup bg = new ButtonGroup(); bg.add(tbCore); bg.add(tbSch);
        tbCore.addActionListener(e -> { cl.show(cards, "core"); tbCore.setBackground(new Color(0x1a3a2a)); tbSch.setBackground(null); });
        tbSch.addActionListener(e -> { cl.show(cards, "scheme"); tbSch.setBackground(new Color(0x1a3a2a)); tbCore.setBackground(null); });
        JPanel tabRow = new JPanel(new FlowLayout(FlowLayout.LEFT, 4, 0));
        tabRow.setBackground(PANEL);
        tabRow.setBorder(BorderFactory.createMatteBorder(0, 0, 1, 0, LINE));
        tabRow.add(tbCore); tabRow.add(tbSch);

        JPanel bottom = new JPanel(new BorderLayout());
        bottom.setBackground(BG);
        bottom.add(buildReadouts(), BorderLayout.NORTH);
        trends = new TrendPanel();
        bottom.add(trends, BorderLayout.CENTER);
        JSplitPane v = new JSplitPane(JSplitPane.VERTICAL_SPLIT, cards, bottom);
        v.setBorder(null); v.setDividerSize(4); v.setResizeWeight(0.62);
        v.setDividerLocation(470);
        left.add(tabRow, BorderLayout.NORTH);
        left.add(v, BorderLayout.CENTER);
        return left;
    }

    private JPanel buildReadouts() {
        JPanel p = new JPanel(new GridLayout(1, 6, 6, 0));
        p.setBackground(BG);
        p.setBorder(BorderFactory.createEmptyBorder(6, 8, 4, 8));
        String[] keys = {"Реакторная мощность, %", "Темп. активной зоны, °C", "Давл. 1-го контура, МПа",
                         "Давл. 2-го контура, МПа", "Эл. мощность, МВт", "Частота сети, Гц"};
        for (int i = 0; i < 6; i++) {
            JPanel card = new JPanel(new BorderLayout());
            card.setBackground(PANEL2);
            card.setBorder(new LineBorder(LINE, 1));
            JLabel k = new JLabel(keys[i]);
            k.setFont(k.getFont().deriveFont(9f)); k.setForeground(DIM);
            bigVal[i] = new JLabel("—", SwingConstants.CENTER);
            bigVal[i].setFont(new Font("Consolas", Font.BOLD, 21));
            bigVal[i].setForeground(GREEN);
            card.add(k, BorderLayout.NORTH);
            card.add(bigVal[i], BorderLayout.CENTER);
            p.add(card);
        }
        return p;
    }
    // ================= ПРАВАЯ КОЛОНКА: управление + сигнализация + журнал =================
    private JPanel buildRight() {
        JPanel right = new JPanel(new BorderLayout());
        right.setBackground(BG);
        JScrollPane ctrlScroll = new JScrollPane(buildControls());
        ctrlScroll.setBorder(null); ctrlScroll.setBackground(PANEL);
        ctrlScroll.getVerticalScrollBar().setUnitIncrement(14);
        JPanel bottom = new JPanel(new GridLayout(3, 1, 0, 6));
        bottom.setBackground(BG);
        bottom.add(buildTasks());
        bottom.add(buildAlarms());
        bottom.add(buildLog());
        JSplitPane v = new JSplitPane(JSplitPane.VERTICAL_SPLIT, ctrlScroll, bottom);
        v.setBorder(null); v.setDividerSize(4); v.setResizeWeight(0.62);
        v.setDividerLocation(400);
        right.add(v, BorderLayout.CENTER);
        return right;
    }

    private JPanel buildControls() {
        JPanel p = new JPanel(new GridBagLayout());
        p.setBackground(PANEL);
        p.setBorder(BorderFactory.createEmptyBorder(8, 8, 8, 8));
        GridBagConstraints g = new GridBagConstraints();
        g.gridx = 0; g.gridy = 0; g.fill = GridBagConstraints.HORIZONTAL;
        g.weightx = 1; g.insets = new Insets(3, 0, 3, 0);
        g.anchor = GridBagConstraints.WEST;

        addSection(p, g, "УПРАВЛЕНИЕ СТЕРЖНЯМИ ОР СУЗ");
        JPanel rodRow = new JPanel(new FlowLayout(FlowLayout.LEFT, 4, 0)); rodRow.setOpaque(false);
        JButton up = new JButton("▲ Извлечь"); styleBtn(up);
        up.addMouseListener(new MouseAdapter() {
            public void mousePressed(MouseEvent e) { R.rodDir = 1; }
            public void mouseReleased(MouseEvent e) { R.rodDir = 0; }
        });
        JButton dn = new JButton("▼ Ввести"); styleBtn(dn);
        dn.addMouseListener(new MouseAdapter() {
            public void mousePressed(MouseEvent e) { R.rodDir = -1; }
            public void mouseReleased(MouseEvent e) { R.rodDir = 0; }
        });
        JButton stop = new JButton("Стоп"); styleBtn(stop);
        stop.addActionListener(e -> R.rodDir = 0);
        rodRow.add(up); rodRow.add(stop); rodRow.add(dn);
        g.gridy++; p.add(rodRow, g);

        JPanel spRow = new JPanel(new FlowLayout(FlowLayout.LEFT, 4, 0)); spRow.setOpaque(false);
        spRow.add(new JLabel("Скорость:"));
        rsSlow = new JButton("Медленно"); rsNorm = new JButton("Нормально"); rsFast = new JButton("Быстро");
        styleBtn(rsSlow); styleBtn(rsNorm); styleBtn(rsFast);
        rsSlow.addActionListener(e -> { R.rodSp = "slow"; refresh(); });
        rsNorm.addActionListener(e -> { R.rodSp = "norm"; refresh(); });
        rsFast.addActionListener(e -> { R.rodSp = "fast"; refresh(); });
        spRow.add(rsSlow); spRow.add(rsNorm); spRow.add(rsFast);
        rodPos = lbl("—", 13, CYAN, true);
        spRow.add(rodPos);
        g.gridy++; p.add(spRow, g);

        addSection(p, g, "СТЕРЖНИ СУЗ: % ИЗВЛЕЧЕНИЯ");
        JPanel pctRow = new JPanel(new FlowLayout(FlowLayout.LEFT, 4, 0)); pctRow.setOpaque(false);
        pctRow.add(new JLabel("Извлечение, %:"));
        rodPctField = new JTextField("50", 4);
        rodPctField.setHorizontalAlignment(JTextField.CENTER);
        rodPctField.setBackground(new Color(0x0d141b));
        rodPctField.setForeground(CYAN);
        rodPctField.setCaretColor(CYAN);
        rodPctField.setBorder(new LineBorder(LINE));
        rodPctField.addActionListener(e -> applyRods());
        pctRow.add(rodPctField);
        JButton applyRods = new JButton("Применить"); styleBtn(applyRods);
        applyRods.addActionListener(e -> applyRods());
        pctRow.add(applyRods);
        g.gridy++; p.add(pctRow, g);

        JPanel selRow = new JPanel(new FlowLayout(FlowLayout.LEFT, 4, 0)); selRow.setOpaque(false);
        JButton selAll = new JButton("Выбрать все"); styleBtn(selAll);
        selAll.addActionListener(e -> { Arrays.fill(rodSel, true); refresh(); core.repaint(); });
        JButton selNone = new JButton("Снять выбор"); styleBtn(selNone);
        selNone.addActionListener(e -> { Arrays.fill(rodSel, false); refresh(); core.repaint(); });
        rodSelLbl = lbl("выбрано: —", 10, AMBER, false);
        selRow.add(selAll); selRow.add(selNone); selRow.add(rodSelLbl);
        g.gridy++; p.add(selRow, g);

        rodMeanLbl = lbl("средн. извлечение: —", 10, DIM, false);
        g.gridy++; p.add(rodMeanLbl, g);

        JLabel rodHint = lbl("Клик по стержню на круге — выбор · 0% = в зоне · 100% = извлечён", 9, DIM, false);
        g.gridy++; p.add(rodHint, g);

        addSection(p, g, "АРК И БОР");
        JToggleButton arkBtn = new JToggleButton("АРК");
        styleBtn(arkBtn); arkBtn.setSelected(true);
        arkBtn.addActionListener(e -> { R.ark = arkBtn.isSelected(); R.addLog("АРК: " + (R.ark ? "ВКЛ" : "ОТКЛ"), R.ark ? "ok" : "warn"); });
        arkLbl = lbl("уставка:", 11, DIM, false);
        pSetSl = new JSlider(0, 110, 100);
        pSetSl.setPreferredSize(new Dimension(110, 20));
        pSetSl.addChangeListener(e -> R.pSet = pSetSl.getValue());
        pSetVal = lbl("100%", 12, CYAN, true);
        JPanel arkRow = new JPanel(new FlowLayout(FlowLayout.LEFT, 4, 0)); arkRow.setOpaque(false);
        arkRow.add(arkBtn); arkRow.add(arkLbl); arkRow.add(pSetSl); arkRow.add(pSetVal);
        g.gridy++; p.add(arkRow, g);

        borSl = new JSlider(0, 100, 60);
        borSl.addChangeListener(e -> R.B = borSl.getValue() / 10.0);
        borVal = lbl("6.0", 12, GREEN, true);
        g.gridy++; p.add(sliderRow("Бор, г/кг", borSl, borVal), g);

        addSection(p, g, "ПЕРВЫЙ КОНТУР — ГЦН И КД");
        JPanel gcnRow = new JPanel(new FlowLayout(FlowLayout.LEFT, 2, 0)); gcnRow.setOpaque(false);
        for (int i = 0; i < 4; i++) {
            final int k = i;
            gcnSw[i] = new ToggleSwitch("ГЦН-" + (k + 1), true, () -> {
                R.gcn[k] = gcnSw[k].on;
                R.addLog("ГЦН-" + (k + 1) + ": " + (R.gcn[k] ? "ВКЛ" : "ОТКЛ"), R.gcn[k] ? "ok" : "warn");
                refresh();
            });
            gcnRow.add(gcnSw[i]);
        }
        g.gridy++; p.add(gcnRow, g);
        gcnLbl = lbl("Расход ГЦН: 100% · насосы 4/4", 10, GREEN, false);
        g.gridy++; p.add(gcnLbl, g);

        JPanel kdRow = new JPanel(new FlowLayout(FlowLayout.LEFT, 6, 0)); kdRow.setOpaque(false);
        makeupSw = new ToggleSwitch("ПОДПИТКА КД", true, () -> {
            R.makeup = makeupSw.on;
            R.addLog("Подпитка КД: " + (R.makeup ? "ВКЛ" : "ОТКЛ"), R.makeup ? "ok" : "warn");
            refresh();
        });
        kdRow.add(makeupSw);
        kdRow.add(lbl("компенсатор давления: подпитка/слив", 9, DIM, false));
        g.gridy++; p.add(kdRow, g);

        addSection(p, g, "БРУ-А (СБРОС ПАРА)");
        JPanel bruRow = new JPanel(new FlowLayout(FlowLayout.LEFT, 4, 0)); bruRow.setOpaque(false);
        bruRow.add(lbl("Режим:", 10, DIM, false));
        bruModeCb = new JComboBox<>(new String[]{"АВТ", "РУЧ", "ОТКЛ"});
        bruModeCb.setSelectedIndex(0);
        bruModeCb.addActionListener(e -> {
            int m = bruModeCb.getSelectedIndex();        // 0 — АВТ, 1 — РУЧ, 2 — ОТКЛ
            R.bruMode = m == 0 ? 1 : m == 1 ? 2 : 0;
            R.bruManual = false;
            R.addLog("БРУ-А: режим " + (R.bruMode == 1 ? "АВТ" : R.bruMode == 2 ? "РУЧ" : "ОТКЛ"), "info");
            refresh();
        });
        bruOpenBtn = new JButton("Открыть"); styleBtn(bruOpenBtn);
        bruOpenBtn.addActionListener(e -> {
            R.bruManual = !R.bruManual;
            R.addLog("БРУ-А ручное: " + (R.bruManual ? "ОТКРЫТ" : "ЗАКРЫТ"), R.bruManual ? "warn" : "ok");
            refresh();
        });
        bruLamp = lbl("ЗАКРЫТ", 9, DIM, true);
        bruRow.add(bruModeCb); bruRow.add(bruOpenBtn); bruRow.add(bruLamp);
        g.gridy++; p.add(bruRow, g);

        addSection(p, g, "ВТОРОЙ КОНТУР — ПИТАНИЕ И ТУРБИНА");
        JPanel fenRow = new JPanel(new FlowLayout(FlowLayout.LEFT, 2, 0)); fenRow.setOpaque(false);
        String[] fenNames = {"ПЭН-1", "ПЭН-2", "МПНА"};
        for (int i = 0; i < 3; i++) {
            final int k = i;
            fenSw[i] = new ToggleSwitch(fenNames[k], k < 2, () -> {
                R.fen[k] = fenSw[k].on;
                R.addLog(fenNames[k] + ": " + (R.fen[k] ? "ВКЛ" : "ОТКЛ"), R.fen[k] ? "ok" : "warn");
                refresh();
            });
            fenRow.add(fenSw[i]);
        }
        g.gridy++; p.add(fenRow, g);
        feedVal = lbl("Питательная вода: 100%", 10, GREEN, false);
        g.gridy++; p.add(feedVal, g);

        govSl = new JSlider(0, 100, 90);
        govSl.addChangeListener(e -> R.gov = govSl.getValue() / 100.0);
        govVal = lbl("90%", 12, GREEN, true);
        g.gridy++; p.add(sliderRow("Турбина: впуск пара", govSl, govVal), g);

        JPanel brkRow = new JPanel(new FlowLayout(FlowLayout.LEFT, 6, 0)); brkRow.setOpaque(false);
        brkSw = new ToggleSwitch("ГЕНЕРАТОР", true, () -> {
            R.breaker = brkSw.on;
            R.addLog("Выключатель генератора: " + (R.breaker ? "ВКЛ" : "ОТКЛ"), R.breaker ? "ok" : "warn");
            refresh();
        });
        brkRow.add(brkSw);
        freqVal = lbl("50.00 Гц", 12, GREEN, true);
        brkRow.add(freqVal);
        g.gridy++; p.add(brkRow, g);

        addSection(p, g, "СЦЕНАРИИ");
        g.gridy++; p.add(btnRow("Работа 100%", () -> loadScenario("full"),
                                  "Пуск блока", () -> loadScenario("start"),
                                  "Останов", () -> loadScenario("stop")), g);

        addSection(p, g, "СОБЫТИЯ (АВАРИИ)");
        g.gridy++; p.add(btnRow("Отказ ГЦН", () -> R.fireEvent("pump"),
                                  "Течь 1 к.", () -> R.fireEvent("loca"),
                                  "Разрыв пар.", () -> R.fireEvent("steam")), g);
        g.gridy++; p.add(btnRow("Ложное АЗ", () -> R.fireEvent("scram"),
                                  "Отказ пит. насоса", () -> R.fireEvent("feed"),
                                  "Случайное", () -> R.fireEvent("random")), g);

        JButton fixBtn = new JButton("Устранить течь / изолировать паропровод");
        styleBtn(fixBtn); fixBtn.addActionListener(e -> R.closeBreak());
        g.gridy++; p.add(fixBtn, g);

        addSection(p, g, "АВАРИЙНАЯ ЗАЩИТА (АЗ)");
        azStatus = lbl("АЗ: ВКЛ", 13, GREEN, true);
        azToggleBtn = new JButton("Отключить АЗ");
        styleBtn(azToggleBtn);
        azToggleBtn.addActionListener(e -> { R.toggleAz(); refresh(); });
        resetAzBtn = new JButton("Взвести АЗ");
        styleBtn(resetAzBtn);
        resetAzBtn.addActionListener(e -> { R.resetScram(); refresh(); });
        JPanel azRow = new JPanel(new FlowLayout(FlowLayout.LEFT, 6, 0)); azRow.setOpaque(false);
        azRow.add(azStatus); azRow.add(azToggleBtn); azRow.add(resetAzBtn);
        g.gridy++; p.add(azRow, g);

        JPanel azKeys = new JPanel(new FlowLayout(FlowLayout.LEFT, 8, 0)); azKeys.setOpaque(false);
        az1Btn = azKeyBtn("АЗ-1"); az2Btn = azKeyBtn("АЗ-2");
        azKeys.add(az1Btn); azKeys.add(az2Btn);
        azKeys.add(lbl("ключи аварийной защиты", 9, DIM, false));
        g.gridy++; g.weighty = 0; p.add(azKeys, g);
        return p;
    }

    private void applyRods() {
        List<Integer> sel = new ArrayList<>();
        for (int k = 0; k < rodSel.length; k++) if (rodSel[k]) sel.add(k);
        if (sel.isEmpty()) { R.addLog("Стержни: не выбран ни один стержень — кликните по стержню на круге", "warn"); return; }
        int pct;
        try { pct = Integer.parseInt(rodPctField.getText().trim()); }
        catch (NumberFormatException ex) { R.addLog("Стержни: некорректный % извлечения", "warn"); return; }
        if (pct < 0 || pct > 100) { R.addLog("Стержни: % извлечения вне диапазона 0–100", "warn"); return; }
        int[] idx = new int[sel.size()];
        for (int i = 0; i < sel.size(); i++) idx[i] = sel.get(i);
        R.setRods(idx, 100 - pct);              // % извлечения → % ввода
        for (int k : idx) rodSel[k] = false;
        refresh();
        core.repaint();
        if (R.ark) R.addLog("Внимание: АРК ВКЛ — регулятор перепозиционирует стержни", "warn");
        StringBuilder sb = new StringBuilder();
        for (int k : idx) sb.append(k + 1).append(' ');
        R.addLog("Стержни " + sb.toString().trim() + ": извлечение " + pct + "% (ввод " + Math.round(100 - pct) + "%)", "ok");
    }

    private JPanel sliderRow(String lbl, JSlider sl, JLabel val) {
        JPanel row = new JPanel(new BorderLayout(6, 0)); row.setOpaque(false);
        JLabel l = new JLabel(lbl); l.setForeground(DIM); l.setPreferredSize(new Dimension(170, 20));
        row.add(l, BorderLayout.WEST);
        row.add(sl, BorderLayout.CENTER);
        row.add(val, BorderLayout.EAST);
        return row;
    }

    private JPanel btnRow(String a, Runnable ra, String b, Runnable rb, String c, Runnable rc) {
        JPanel row = new JPanel(new FlowLayout(FlowLayout.LEFT, 4, 0)); row.setOpaque(false);
        JButton ba = new JButton(a); styleBtn(ba); ba.addActionListener(e -> ra.run()); row.add(ba);
        JButton bb = new JButton(b); styleBtn(bb); bb.addActionListener(e -> rb.run()); row.add(bb);
        JButton bc = new JButton(c); styleBtn(bc); bc.addActionListener(e -> rc.run()); row.add(bc);
        return row;
    }

    private void addSection(JPanel p, GridBagConstraints g, String title) {
        JLabel l = new JLabel(title);
        l.setFont(l.getFont().deriveFont(Font.BOLD, 10f));
        l.setForeground(CYAN);
        l.setBorder(BorderFactory.createMatteBorder(0, 0, 1, 0, LINE));
        g.gridy++; p.add(l, g);
    }

    // ================= ЗАДАНИЯ =================
    private JPanel buildTasks() {
        JPanel p = new JPanel(new BorderLayout());
        p.setBackground(PANEL);
        p.setBorder(BorderFactory.createTitledBorder(
                BorderFactory.createLineBorder(LINE), "ЗАДАНИЯ ОПЕРАТОРА",
                0, 0, new Font("Segoe UI", Font.BOLD, 10), CYAN));
        tasksBox = new JPanel();
        tasksBox.setLayout(new BoxLayout(tasksBox, BoxLayout.Y_AXIS));
        tasksBox.setBackground(PANEL2);
        p.add(new JScrollPane(tasksBox), BorderLayout.CENTER);
        return p;
    }

    private void updateTasks() {
        StringBuilder sb = new StringBuilder();
        for (Reactor.Task t : R.tasks) sb.append(t.state).append('|').append(t.text).append(';');
        String s = sb.toString();
        if (s.equals(taskSig)) return;
        taskSig = s;
        tasksBox.removeAll();
        if (R.tasks.isEmpty()) {
            tasksBox.add(lbl("○ Нет активных заданий", 11, DIM, false));
        } else {
            for (Reactor.Task t : R.tasks) {
                JPanel row = new JPanel(new FlowLayout(FlowLayout.LEFT, 6, 1));
                row.setOpaque(false);
                String ic = t.state == 1 ? "✔" : t.state == 2 ? "✘" : "○";
                Color c = t.state == 1 ? GREEN : t.state == 2 ? RED : DIM;
                JLabel il = lbl(ic, 12, c, true);
                JLabel tl = lbl(t.text, 11, t.state == 1 ? TXT : (t.state == 2 ? RED : new Color(0x9fb4c6)), false);
                row.add(il); row.add(tl);
                tasksBox.add(row);
            }
        }
        tasksBox.revalidate();
        tasksBox.repaint();
    }

    // ================= СИГНАЛИЗАЦИЯ =================
    private JPanel buildAlarms() {
        JPanel p = new JPanel(new BorderLayout());
        p.setBackground(PANEL);
        p.setBorder(BorderFactory.createTitledBorder(
                BorderFactory.createLineBorder(LINE), "Аварийная сигнализация",
                0, 0, new Font("Segoe UI", Font.BOLD, 10), CYAN));
        alarmList = new JList<>(alarmModel);
        JList<Reactor.Alarm> list = alarmList;
        list.setBackground(PANEL2);
        list.setCellRenderer((li, alarm, idx, sel, foc) -> {
            JLabel lb = new JLabel(fmtTime(alarm.t) + "  " + alarm.text);
            lb.setOpaque(true);
            lb.setBackground(alarm.warn ? new Color(0x1a140a) : new Color(0x160b0b));
            lb.setForeground(alarm.warn ? AMBER : RED);
            if (alarm.ack) lb.setForeground(DIM);
            return lb;
        });
        p.add(new JScrollPane(list), BorderLayout.CENTER);
        JButton ack = new JButton("Квитировать"); styleBtn(ack);
        ack.addActionListener(e -> R.ackAll());
        p.add(ack, BorderLayout.SOUTH);
        return p;
    }

    private JPanel buildLog() {
        JPanel p = new JPanel(new BorderLayout());
        p.setBackground(PANEL);
        p.setBorder(BorderFactory.createTitledBorder(
                BorderFactory.createLineBorder(LINE), "Журнал оператора",
                0, 0, new Font("Segoe UI", Font.BOLD, 10), CYAN));
        logPane = new JTextPane();
        logPane.setEditable(false);
        logPane.setBackground(PANEL2);
        logPane.setFont(new Font("Consolas", Font.PLAIN, 11));
        p.add(new JScrollPane(logPane), BorderLayout.CENTER);
        return p;
    }
    // ================= ОБЗОР РЕАКТОРА (сфера с ТК) =================
    class CorePanel extends JPanel {
        CorePanel() {
            setBackground(PANEL);
            addMouseListener(new MouseAdapter() {
                public void mousePressed(MouseEvent e) {
                    int k = rodAt(e.getX(), e.getY());
                    if (k >= 0) { rodSel[k] = !rodSel[k]; refresh(); repaint(); }
                }
            });
        }
        private double cellSize() {
            return Math.min(Math.min(getWidth() * 0.58 / Reactor.CORE_N, getHeight() * 0.72 / Reactor.CORE_N), 46);
        }
        private double coreCx() { return getWidth() * 0.38; }
        private double coreCy() { return getHeight() * 0.53; }
        private double coreRr() {
            return Math.min(Math.min(cellSize() * (Reactor.CORE_N / 2.0 + 0.6) * 1.25, getHeight() * 0.45), getWidth() * 0.32);
        }
        private int rodAt(int mx, int my) {
            double cell = cellSize(), cx = coreCx(), cy = coreCy();
            for (int k = 0; k < Reactor.ROD_N; k++) {
                int gi = 1 + 3 * (k / 3), gj = 1 + 3 * (k % 3);
                double x = cx + (gj - (Reactor.CORE_N - 1) / 2.0) * cell;
                double y = cy + (gi - (Reactor.CORE_N - 1) / 2.0) * cell;
                if (Math.hypot(mx - x, my - y) < cell * 0.45) return k;
            }
            return -1;
        }
        private int selCount() { int c = 0; for (boolean b : rodSel) if (b) c++; return c; }

        private Color lerp(Color a, Color b, double f) {
            return new Color(
                    (int) (a.getRed() + (b.getRed() - a.getRed()) * f),
                    (int) (a.getGreen() + (b.getGreen() - a.getGreen()) * f),
                    (int) (a.getBlue() + (b.getBlue() - a.getBlue()) * f));
        }
        private Color tempColor(double t) {
            double[][] stops = {
                    {300, 0x1e3f66}, {600, 0x2f6fbf}, {900, 0x3fb0e0}, {1200, 0x3ddc84},
                    {1600, 0xd8e04d}, {2000, 0xff9a3d}, {2400, 0xff5a5a}, {2800, 0xffc8c8}};
            if (t <= stops[0][0]) return new Color((int) stops[0][1]);
            for (int i = 1; i < stops.length; i++) {
                if (t <= stops[i][0]) {
                    double f = (t - stops[i - 1][0]) / (stops[i][0] - stops[i - 1][0]);
                    return lerp(new Color((int) stops[i - 1][1]), new Color((int) stops[i][1]), f);
                }
            }
            return new Color((int) stops[stops.length - 1][1]);
        }

        public void paintComponent(Graphics gr) {
            super.paintComponent(gr);
            Graphics2D g = (Graphics2D) gr;
            g.setRenderingHint(RenderingHints.KEY_ANTIALIASING, RenderingHints.VALUE_ANTIALIAS_ON);
            int w = getWidth(), h = getHeight();
            int n = Reactor.CORE_N;
            double cell = cellSize();
            double cx = coreCx(), cy = coreCy();
            double Rr = coreRr();

            // сосуд реактора (шар, вид сверху)
            RadialGradientPaint shell = new RadialGradientPaint((float) cx, (float) cy, (float) Rr,
                    new float[]{0f, 0.75f, 1f},
                    new Color[]{new Color(0x10161d), new Color(0x1c2a36), new Color(0x2a3d4d)});
            g.setPaint(shell);
            g.fillOval((int) (cx - Rr), (int) (cy - Rr), (int) (2 * Rr), (int) (2 * Rr));
            g.setColor(LINE); g.setStroke(new BasicStroke(3));
            g.drawOval((int) (cx - Rr), (int) (cy - Rr), (int) (2 * Rr), (int) (2 * Rr));
            g.setColor(DIM); g.setStroke(new BasicStroke(1));
            g.drawOval((int) (cx - Rr * 1.08), (int) (cy - Rr * 1.08), (int) (2 * Rr * 1.08), (int) (2 * Rr * 1.08));

            // ТК (кассеты) с индивидуальной температурой ТОПЛИВА
            double maxT = -1e9, minT = 1e9; int mxi = 0, mxj = 0;
            for (int i = 0; i < n; i++) for (int j = 0; j < n; j++) {
                double d = Math.hypot(j - (n - 1) / 2.0, i - (n - 1) / 2.0) / ((n - 1) / 2.0);
                if (d > 1.02) continue;
                double t = R.fuelTemp(i, j);
                double m = R.melt[i][j];
                if (t > maxT) { maxT = t; mxi = i; mxj = j; }
                if (t < minT) minT = t;
                double x = cx + (j - (n - 1) / 2.0) * cell, y = cy + (i - (n - 1) / 2.0) * cell;
                double s = cell * 0.76;
                g.setColor(tempColor(t));
                g.fillRoundRect((int) (x - s / 2), (int) (y - s / 2), (int) s, (int) s, 6, 6);
                if (m > 0.01) {                              // расплавление: затемнение к чёрному
                    g.setComposite(AlphaComposite.getInstance(AlphaComposite.SRC_OVER, (float) Math.min(0.92, m)));
                    g.setColor(Color.BLACK);
                    g.fillRoundRect((int) (x - s / 2), (int) (y - s / 2), (int) s, (int) s, 6, 6);
                    g.setComposite(AlphaComposite.getInstance(AlphaComposite.SRC_OVER, 1f));
                }
                if (m >= 1) {                                // ТК расплавлена полностью
                    g.setColor(RED); g.setStroke(new BasicStroke(2));
                    g.drawRoundRect((int) (x - s / 2), (int) (y - s / 2), (int) s, (int) s, 6, 6);
                    g.setStroke(new BasicStroke(1));
                }
            }
            // рамка самой горячей ТК
            g.setColor(Color.WHITE); g.setStroke(new BasicStroke(2));
            double hx = cx + (mxj - (n - 1) / 2.0) * cell, hy = cy + (mxi - (n - 1) / 2.0) * cell;
            g.drawRoundRect((int) (hx - cell * 0.5), (int) (hy - cell * 0.5), (int) cell, (int) cell, 6, 6);
            // цифры температуры в каждой ТК
            g.setFont(new Font("Consolas", Font.PLAIN, (int) Math.max(8, cell * 0.30)));
            for (int i = 0; i < n; i++) for (int j = 0; j < n; j++) {
                double d = Math.hypot(j - (n - 1) / 2.0, i - (n - 1) / 2.0) / ((n - 1) / 2.0);
                if (d > 1.02) continue;
                double x = cx + (j - (n - 1) / 2.0) * cell, y = cy + (i - (n - 1) / 2.0) * cell;
                String s = String.valueOf(Math.round(R.fuelTemp(i, j)));
                g.setColor(R.fuelTemp(i, j) > 2200 ? Color.WHITE : new Color(0x0a0e12));
                g.drawString(s, (int) (x - g.getFontMetrics().stringWidth(s) / 2.0),
                        (int) (y + g.getFontMetrics().getAscent() / 2.0));
            }

            // стержни ОР СУЗ: позиции 3×3 поверх зоны; клик — выбор, под стержнем % извлечения
            for (int k = 0; k < Reactor.ROD_N; k++) {
                int gi = 1 + 3 * (k / 3), gj = 1 + 3 * (k % 3);
                double x = cx + (gj - (n - 1) / 2.0) * cell, y = cy + (gi - (n - 1) / 2.0) * cell;
                double rr = cell * 0.34;
                double ext = R.rodIns == null ? 100 - Reactor.clamp(R.I, 0, 100) : R.rodExtract(k);
                // тело стержня
                g.setColor(new Color(0x0d141b));
                g.fillOval((int) (x - rr), (int) (y - rr), (int) (2 * rr), (int) (2 * rr));
                g.setColor(ext > 80 ? new Color(0x2f6fbf) : ext > 20 ? new Color(0x5f7385) : new Color(0x8a2f2f));
                g.setStroke(new BasicStroke(2));
                g.drawOval((int) (x - rr), (int) (y - rr), (int) (2 * rr), (int) (2 * rr));
                // номер стержня
                g.setFont(new Font("Consolas", Font.BOLD, (int) Math.max(8, cell * 0.24)));
                g.setColor(TXT);
                String num = String.valueOf(k + 1);
                g.drawString(num, (int) (x - g.getFontMetrics().stringWidth(num) / 2.0),
                        (int) (y + g.getFontMetrics().getAscent() / 2.0 - 2));
                // % извлечения под стержнем
                g.setFont(new Font("Consolas", Font.PLAIN, (int) Math.max(7, cell * 0.19)));
                g.setColor(DIM);
                String pct = Math.round(ext) + "%";
                g.drawString(pct, (int) (x - g.getFontMetrics().stringWidth(pct) / 2.0), (int) (y + rr + 9));
                // выделение выбранного
                if (rodSel[k]) {
                    g.setColor(AMBER); g.setStroke(new BasicStroke(3));
                    g.drawOval((int) (x - rr - 3), (int) (y - rr - 3), (int) (2 * rr + 6), (int) (2 * rr + 6));
                    g.setStroke(new BasicStroke(1));
                }
            }

            // заголовок и статистика
            g.setFont(new Font("Segoe UI", Font.BOLD, 11));
            g.setColor(CYAN);
            g.drawString("ОБЗОР РЕАКТОРА — ТОПЛИВО АКТИВНОЙ ЗОНЫ (вид сверху)", (int) (w * 0.03), (int) (h * 0.06));
            g.setFont(new Font("Consolas", Font.PLAIN, 11));
            g.setColor(TXT);
            g.drawString("Макс. топливо: " + Math.round(maxT) + "°C   Мин: " + Math.round(minT) + "°C   плавление ≥ " + Math.round(Reactor.MELT_T) + "°C",
                    (int) (w * 0.03), (int) (h * 0.11));
            double mTot = R.meltTotal();
            g.drawString("Мощность: " + Reactor.f1(R.P) + "%   Темп. зоны: " + Reactor.f1(R.Tcore) + "°C   Расплавлено: " + R.meltedCount() + " ТК"
                    + (mTot > 0.01 ? " (" + Math.round(mTot * 100 / (Reactor.CORE_N * Reactor.CORE_N)) + "% зоны)" : ""),
                    (int) (w * 0.03), (int) (h * 0.16));
            g.drawString("Клик по стержню на круге — выбор · выбрано: " + selCount() + " из " + Reactor.ROD_N,
                    (int) (w * 0.03), (int) (h * 0.21));

            // радиальный профиль температуры
            int px0 = (int) (w * 0.66), px1 = (int) (w * 0.96), py0 = (int) (h * 0.16), py1 = (int) (h * 0.52);
            g.setColor(new Color(0x1a2733));
            g.fillRoundRect(px0 - 8, py0 - 18, px1 - px0 + 16, py1 - py0 + 40, 8, 8);
            g.setFont(new Font("Segoe UI", Font.PLAIN, 10));
            g.setColor(DIM);
            g.drawString("Радиальный профиль температуры топлива", px0, py0 - 6);
            double tLo = 300, tHi = 2800;
            g.setColor(LINE);
            g.drawLine(px0, py0, px0, py1); g.drawLine(px0, py1, px1, py1);
            g.setColor(new Color(0xff9a3d));
            g.setStroke(new BasicStroke(2));
            int prevX = px0, prevY = py1;
            for (int k = 0; k < 40; k++) {
                double f = k / 39.0;                              // 0 — центр, 1 — край
                int j = (int) Math.round((n - 1) / 2.0 * f);
                double t = R.fuelTemp((n - 1) / 2, (n - 1) / 2 + j);
                int x = px0 + (int) (f * (px1 - px0));
                int y = py1 - (int) ((py1 - py0) * Reactor.clamp((t - tLo) / (tHi - tLo), 0, 1));
                g.drawLine(prevX, prevY, x, y);
                prevX = x; prevY = y;
            }
            g.setStroke(new BasicStroke(1));
            g.setFont(new Font("Consolas", Font.PLAIN, 9));
            g.setColor(DIM);
            g.drawString("центр", px0, py1 + 14); g.drawString("край", px1 - 20, py1 + 14);
            g.drawString("300°C", px0 - 10, py1);

            // легенда шкалы
            int lx0 = (int) (w * 0.66), lx1 = (int) (w * 0.96), ly = (int) (h * 0.72);
            g.setFont(new Font("Segoe UI", Font.PLAIN, 9));
            g.setColor(DIM);
            g.drawString("Шкала температуры топлива ТК, °C", lx0, ly - 6);
            for (int x = lx0; x < lx1; x++) {
                double f = (x - lx0) / (double) (lx1 - lx0);
                g.setColor(tempColor(300 + f * 2500));
                g.drawLine(x, ly, x, ly + 14);
            }
            g.setColor(DIM);
            g.drawString("300", lx0 - 2, ly + 26);
            g.drawString("2800", lx1 - 24, ly + 26);
        }
    }

    // ================= СХЕМА (кастомная отрисовка) =================
    class SchemePanel extends JPanel {
        private double rotorAngle = 0;
        SchemePanel() { setBackground(PANEL); }

        public void paintComponent(Graphics gr) {
            super.paintComponent(gr);
            Graphics2D g = (Graphics2D) gr;
            g.setRenderingHint(RenderingHints.KEY_ANTIALIASING, RenderingHints.VALUE_ANTIALIAS_ON);
            int w = getWidth(), h = getHeight();
            double s = Math.min(w / 1000.0, h / 560.0);
            g.translate((w - 1000 * s) / 2, (h - 560 * s) / 2);
            g.scale(s, s);
            double glow = Reactor.clamp(R.P / 130, 0, 1);
            double rpm = R.breaker ? 3000 * Reactor.clamp(R.gov > 0 ? Math.min(1, R.P2 / 6.4) : 0.4, 0, 1) : 0;
            rotorAngle += rpm / 60 * 360 * 0.1;

            // --- реактор ---
            g.setColor(new Color(0x15202b));
            g.fillRoundRect(140, 200, 150, 210, 16, 16);
            g.setColor(LINE); g.drawRoundRect(140, 200, 150, 210, 16, 16);
            // свечение активной зоны
            if (glow > 0.01) {
                g.setComposite(AlphaComposite.getInstance(AlphaComposite.SRC_OVER, (float) glow));
                RadialGradientPaint rg = new RadialGradientPaint(215, 300, 60,
                        new float[]{0f, 0.5f, 1f},
                        new Color[]{new Color(255, 210, 122), new Color(255, 122, 26), new Color(255, 61, 0, 0)});
                g.setPaint(rg);
                g.fillRoundRect(176, 251, 78, 98, 8, 8);
                g.setComposite(AlphaComposite.getInstance(AlphaComposite.SRC_OVER, 1f));
            }
            g.setColor(R.P > 110 ? RED : new Color(0x6b4a26));
            g.drawRoundRect(180, 255, 70, 90, 6, 6);
            g.setColor(DIM);
            g.drawString("РЕАКТОР", 215 - g.getFontMetrics().stringWidth("РЕАКТОР") / 2, 130);
            g.drawString("АКТИВНАЯ ЗОНА", 215 - g.getFontMetrics().stringWidth("АКТИВНАЯ ЗОНА") / 2, 310);

            // --- компенсатор давления ---
            g.setColor(new Color(0x15202b)); g.fillRoundRect(60, 130, 46, 120, 6, 6);
            g.setColor(LINE); g.drawRoundRect(60, 130, 46, 120, 6, 6);
            g.setColor(new Color(0x2b5a8f));
            g.fillRect(80, 178, 6, (int) Reactor.clamp((R.P1 - 14) / 4 * 40, 0, 40));
            g.setColor(DIM); g.drawString("КД", 83 - g.getFontMetrics().stringWidth("КД") / 2, 270);
            pipe(g, 83, 250, 83, 290, new Color(0x33485c)); pipe(g, 83, 290, 140, 290, new Color(0x33485c));

            // --- трубы 1 контура ---
            Color hot = R.Tcore > 345 ? RED : new Color(0xc46a2a);
            Color cold = new Color(0x3f7fbf);
            pipe(g, 215, 215, 360, 215, hot); pipe(g, 360, 215, 360, 235, hot);
            pipe(g, 215, 395, 360, 395, cold); pipe(g, 360, 395, 360, 375, cold);
            // ГЦН
            g.setColor(new Color(0x15202b)); g.fillOval(280, 375, 40, 40);
            g.setColor(LINE); g.drawOval(280, 375, 40, 40);
            String gcnTxt = "ГЦН " + R.gcnCount() + "/4";
            g.setColor(R.gcnCount() < 4 ? AMBER : DIM);
            g.drawString(gcnTxt, 300 - g.getFontMetrics().stringWidth(gcnTxt) / 2, 400);
            // поток
            if (R.pump > 0.05) flow(g, 215, 395, 345, 395, R.pump);
            if (R.pump > 0.05) flow(g, 215, 215, 345, 215, R.pump);

            // --- парогенератор ---
            g.setColor(new Color(0x15202b)); g.fillRoundRect(360, 120, 200, 150, 20, 20);
            g.setColor(LINE); g.drawRoundRect(360, 120, 200, 150, 20, 20);
            double sg = Reactor.clamp(R.P / 120, 0, 1);
            if (sg > 0.01) {
                g.setComposite(AlphaComposite.getInstance(AlphaComposite.SRC_OVER, (float) sg * 0.5f));
                g.setColor(new Color(0xff9a3d));
                g.fillRoundRect(368, 240, 184, 22, 8, 8);
                g.setComposite(AlphaComposite.getInstance(AlphaComposite.SRC_OVER, 1f));
            }
            g.setColor(DIM); g.drawString("ПАРОГЕНЕРАТОР", 460 - g.getFontMetrics().stringWidth("ПАРОГЕНЕРАТОР") / 2, 150);
            g.drawString("ПГ " + Reactor.f1(R.Tsg) + "°C", 460 - g.getFontMetrics().stringWidth("ПГ " + Reactor.f1(R.Tsg) + "°C") / 2, 292);
            g.drawString("Уровень ПГ: " + Math.round(R.level) + "%", 520 - g.getFontMetrics().stringWidth("Уровень ПГ: " + Math.round(R.level) + "%") / 2, 315);

            // --- пар ---
            pipe(g, 520, 120, 520, 80, new Color(0x33485c)); pipe(g, 520, 80, 620, 80, new Color(0x33485c));
            pipe(g, 400, 120, 400, 80, new Color(0x33485c)); pipe(g, 400, 80, 470, 80, new Color(0x33485c));
            if (R.P2 > 0.5) flow(g, 400, 84, 600, 84, R.gov * R.P2 / 6.4);
            g.setColor(DIM); g.drawString("ПАР", 560 - g.getFontMetrics().stringWidth("ПАР") / 2, 70);
            g.setColor(R.relS ? RED : new Color(0x7a2f2f));
            g.fillRect(516, 72, 8, 16);
            g.drawString("ГПЗ", 540 - g.getFontMetrics().stringWidth("ГПЗ") / 2, 112);

            // --- турбина ---
            g.setColor(new Color(0x15202b)); g.fillRoundRect(620, 130, 120, 70, 12, 12);
            g.setColor(LINE); g.drawRoundRect(620, 130, 120, 70, 12, 12);
            AffineTransform save = g.getTransform();
            g.translate(680, 165); g.rotate(Math.toRadians(rotorAngle));
            g.setColor(new Color(0x4f6a80)); g.setStroke(new BasicStroke(4));
            for (int i = 0; i < 6; i++) {
                g.rotate(Math.toRadians(60));
                g.drawLine(-34, 0, 34, 0);
            }
            g.setTransform(save);
            g.setStroke(new BasicStroke(1));
            g.setColor(DIM); g.drawString("ТУРБИНА", 680 - g.getFontMetrics().stringWidth("ТУРБИНА") / 2, 120);
            g.drawString("3000 об/мин", 680 - g.getFontMetrics().stringWidth("3000 об/мин") / 2, 115);

            // --- генератор ---
            g.setColor(new Color(0x15202b)); g.fillRoundRect(760, 145, 90, 40, 8, 8);
            g.setColor(LINE); g.drawRoundRect(760, 145, 90, 40, 8, 8);
            g.setColor(DIM); g.drawString("ГЕНЕРАТОР", 805 - g.getFontMetrics().stringWidth("ГЕНЕРАТОР") / 2, 168);
            g.setColor(R.MWe > 10 ? GREEN : DIM);
            g.drawString(Math.round(R.MWe) + " МВт", 805 - g.getFontMetrics().stringWidth(Math.round(R.MWe) + " МВт") / 2, 205);
            pipe(g, 760, 165, 712, 165, new Color(0x31455a));
            pipe(g, 850, 165, 895, 165, new Color(0x31455a));
            pipe(g, 895, 165, 895, 120, new Color(0x31455a)); pipe(g, 895, 120, 930, 120, new Color(0x31455a));
            g.setColor(DIM); g.drawString(R.breaker ? "СЕТЬ" : "ОТКЛЮЧЕНО", 905, 140);
            g.setColor(R.breaker ? new Color(0x1f4a2f) : new Color(0x5a1a1a));
            g.fillOval(843, 158, 14, 14);

            // --- конденсатор и охл. вода ---
            g.setColor(new Color(0x15202b)); g.fillRoundRect(620, 330, 200, 70, 12, 12);
            g.setColor(LINE); g.drawRoundRect(620, 330, 200, 70, 12, 12);
            g.setColor(DIM); g.drawString("КОНДЕНСАТОР", 720 - g.getFontMetrics().stringWidth("КОНДЕНСАТОР") / 2, 372);
            pipe(g, 660, 200, 660, 330, new Color(0x33485c)); pipe(g, 740, 200, 740, 330, new Color(0x33485c));
            pipe(g, 640, 420, 640, 460, new Color(0x2a3c4d)); pipe(g, 640, 460, 760, 460, new Color(0x2a3c4d)); pipe(g, 760, 460, 760, 400, new Color(0x2a3c4d));
            flow(g, 645, 452, 755, 452, 1);
            g.setColor(DIM); g.drawString("ОХЛ. ВОДА (река)", 700 - g.getFontMetrics().stringWidth("ОХЛ. ВОДА (река)") / 2, 440);

            // --- питательная вода ---
            pipe(g, 560, 380, 560, 320, cold); pipe(g, 560, 320, 600, 320, cold); pipe(g, 600, 320, 600, 150, cold);
            if (R.feed > 0.05) flow(g, 560, 375, 560, 325, R.feed);
            g.setColor(new Color(0x15202b)); g.fillOval(573, 363, 34, 34);
            g.setColor(LINE); g.drawOval(573, 363, 34, 34);
            g.setColor(DIM); g.drawString("ПН", 590 - g.getFontMetrics().stringWidth("ПН") / 2, 385);

            g.setColor(TXT);
            g.drawString("Т зоны " + Reactor.f1(R.Tcore) + "°C", 60, 460);
        }

        private void pipe(Graphics2D g, int x1, int y1, int x2, int y2, Color c) {
            g.setColor(c); g.setStroke(new BasicStroke(7, BasicStroke.CAP_ROUND, BasicStroke.JOIN_ROUND));
            g.drawLine(x1, y1, x2, y2);
            g.setStroke(new BasicStroke(1));
        }
        private void flow(Graphics2D g, int x1, int y1, int x2, int y2, double rate) {
            if (rate <= 0) return;
            g.setColor(new Color(111, 195, 255, 200));
            g.setStroke(new BasicStroke(3, BasicStroke.CAP_ROUND, BasicStroke.JOIN_ROUND,
                    1, new float[]{3, 14}, (float) (R.time * 40 * rate)));
            g.drawLine(x1, y1, x2, y2);
            g.setStroke(new BasicStroke(1));
        }
    }

    // ================= ГРАФИКИ =================
    class TrendPanel extends JPanel {
        TrendPanel() { setBackground(PANEL); }
        public void paintComponent(Graphics gr) {
            super.paintComponent(gr);
            Graphics2D g = (Graphics2D) gr;
            g.setRenderingHint(RenderingHints.KEY_ANTIALIASING, RenderingHints.VALUE_ANTIALIAS_ON);
            int w = getWidth(), h = getHeight();
            g.setColor(DIM); g.setFont(new Font("Consolas", Font.PLAIN, 9));
            g.drawString("Мощность %", 10, 12); g.setColor(GREEN); g.drawLine(66, 9, 90, 9);
            g.setColor(DIM); g.drawString("Темп. °C", 95, 12); g.setColor(new Color(0xff7a1a)); g.drawLine(130, 9, 154, 9);
            g.setColor(DIM); g.drawString("Давл.1 МПа", 159, 12); g.setColor(CYAN); g.drawLine(200, 9, 224, 9);
            g.setColor(DIM); g.drawString("МВт(э)", 229, 12); g.setColor(YELLOW); g.drawLine(262, 9, 286, 9);
            g.setColor(new Color(0x1a2733));
            for (int i = 0; i <= 4; i++) {
                int y = h - 14 - (h - 30) * i / 4;
                g.drawLine(4, y, w - 4, y);
            }
            List<double[]> tr = R.trends;
            if (tr.size() < 2) return;
            double[][] series = {
                {250, 0, GREEN.getRGB()},
                {420, 0, new Color(0xff7a1a).getRGB()},
                {25, 0, CYAN.getRGB()},
                {1000, 0, YELLOW.getRGB()}
            };
            int n = tr.size();
            for (int si = 0; si < 4; si++) {
                double sc = series[si][0];
                g.setColor(new Color((int) series[si][2]));
                g.setStroke(new BasicStroke(1.6f));
                g.drawPolyline(xpts(n, w), ypts(tr, si, sc, h), n);
            }
            g.setStroke(new BasicStroke(1));
        }
        private int[] xpts(int n, int w) {
            int[] x = new int[n];
            for (int i = 0; i < n; i++) x[i] = 4 + (w - 8) * i / Math.max(1, n - 1);
            return x;
        }
        private int[] ypts(List<double[]> tr, int si, double sc, int h) {
            int n = tr.size();
            int[] y = new int[n];
            for (int i = 0; i < n; i++) {
                double v = tr.get(i)[si];
                y[i] = h - 14 - (int) ((h - 30) * Reactor.clamp(v / sc, 0, 1));
            }
            return y;
        }
    }

    // ================= ОБНОВЛЕНИЕ =================
    private void tick() {
        double dt = 0.1 * R.rate;
        R.advance(dt);
        refresh();
        updateAlarms();
        updateLog();
        scheme.repaint();
        core.repaint();
        trends.repaint();
        if (R.dead && !overlayShown) {
            overlayShown = true;
            SwingUtilities.invokeLater(() -> JOptionPane.showMessageDialog(scheme,
                "Произошло повреждение активной зоны. Блок остановлен.\n" +
                "Загрузите сценарий для продолжения тренировки.",
                "⚠ АВАРИЙНАЯ ОСТАНОВКА БЛОКА", JOptionPane.ERROR_MESSAGE));
        }
    }

    private void refresh() {
        clock.setText(fmtTime(R.time));
        hdrMWe.setText(String.valueOf(Math.round(R.MWe)));
        // крупные индикаторы
        setBig(0, Reactor.f1(R.P), col(R.P, 90, 110));
        setBig(1, Reactor.f1(R.Tcore), col(R.Tcore, 305, 345));
        setBig(2, Reactor.f2(R.P1), col(R.P1, 15.4, 18.2));
        setBig(3, Reactor.f2(R.P2), col(R.P2, 6.2, 7.4));
        setBig(4, String.valueOf(Math.round(R.MWe)), R.gov > 0 ? GREEN : DIM);
        setBig(5, R.breaker ? Reactor.f2(R.f) : "—", GREEN);
        // статус
        Color sc = GREEN; String st = "РАБОТА";
        if (R.dead) { sc = RED; st = "АВАРИЯ"; }
        else if (R.scram) { sc = RED; st = "АЗ СРАБОТАЛА"; }
        else if (R.azState == 2) { sc = AMBER; st = "АЗ ОТКЛЮЧЕНА"; }
        else if (!R.alarms.isEmpty()) { sc = AMBER; st = "ПРЕДУПРЕЖДЕНИЕ"; }
        lamp.setForeground(sc); statusText.setText(st); statusText.setForeground(sc);
        // элементы управления
        if (gcnLbl != null) {
            gcnLbl.setText("Расход ГЦН: " + Math.round(R.pump * 100) + "% · насосы " + R.gcnCount() + "/4");
            gcnLbl.setForeground(R.gcnCount() < 4 ? AMBER : GREEN);
        }
        feedVal.setText("Питательная вода: " + Math.round(R.feed * 100) + "%");
        govVal.setText(Math.round(R.gov * 100) + "%");
        borVal.setText(Reactor.f1(R.B));
        pSetVal.setText((int) R.pSet + "%");
        rodPos.setText("извлечение " + Reactor.f1(R.rodExtractMean()) + "%");
        rodPos.setForeground(R.rodExtractMean() < 15 ? RED : CYAN);
        if (rodSel != null) {
            StringBuilder selB = new StringBuilder();
            int selN = 0;
            for (int k = 0; k < rodSel.length; k++) if (rodSel[k]) { selN++; selB.append(k + 1).append(','); }
            if (selN > 0) selB.setLength(selB.length() - 1);
            rodSelLbl.setText("выбрано: " + (selN == 0 ? "—" : selB.toString()));
            rodMeanLbl.setText("средн. извлечение: " + Math.round(R.rodExtractMean()) + "%");
            rodMeanLbl.setForeground(R.rodExtractMean() < 5 ? RED : DIM);
        }
        freqVal.setText(R.breaker ? Reactor.f2(R.f) + " Гц" : "СЕТЬ ОТКЛ");
        freqVal.setForeground(R.breaker ? GREEN : RED);
        Color azCol = R.scram ? new Color(0x8a1515) : new Color(0x5a1010);
        az1Btn.setBackground(azCol); az2Btn.setBackground(azCol);
        azStatus.setText(R.azState == 0 ? "АЗ: ВКЛ" : R.azState == 1 ? "АЗ: СРАБОТАЛА" : "АЗ: ОТКЛЮЧЕНА");
        azStatus.setForeground(R.azState == 0 ? GREEN : R.azState == 1 ? RED : AMBER);
        azToggleBtn.setText(R.azState == 2 ? "Включить АЗ" : "Отключить АЗ");
        azToggleBtn.setEnabled(R.azState != 1);
        resetAzBtn.setEnabled(R.azState == 1 || R.azState == 2);
        updateTasks();
        syncSwitches();
        if (bruLamp != null) {
            boolean bo = R.bruOpen();
            bruLamp.setText(bo ? "ОТКРЫТ" : "ЗАКРЫТ");
            bruLamp.setForeground(bo ? AMBER : DIM);
        }
        if (bruOpenBtn != null) bruOpenBtn.setEnabled(R.bruMode == 2);
        // кнопки скорости
        rsSlow.setBackground("slow".equals(R.rodSp) ? new Color(0x1a3a2a) : null);
        rsNorm.setBackground("norm".equals(R.rodSp) ? new Color(0x1a3a2a) : null);
        rsFast.setBackground("fast".equals(R.rodSp) ? new Color(0x1a3a2a) : null);
        // синхронизация ползунков (кроме тех, что под фокусом)
        if (!govSl.hasFocus()) govSl.setValue((int) (R.gov * 100));
        if (!borSl.hasFocus()) borSl.setValue((int) (R.B * 10));
        if (!pSetSl.hasFocus()) pSetSl.setValue((int) R.pSet);
    }

    /** Синхронизировать переключатели с моделью (сценарии, события) без срабатывания обработчиков. */
    private void syncSwitches() {
        if (gcnSw[0] == null) return;
        for (int i = 0; i < 4; i++) gcnSw[i].setOn(R.gcn[i], false);
        for (int i = 0; i < 3; i++) fenSw[i].setOn(R.fen[i], false);
        makeupSw.setOn(R.makeup, false);
        brkSw.setOn(R.breaker, false);
        if (bruModeCb != null) {
            int idx = R.bruMode == 1 ? 0 : R.bruMode == 2 ? 1 : 2;
            if (bruModeCb.getSelectedIndex() != idx) bruModeCb.setSelectedIndex(idx);
        }
    }

    private JButton azKeyBtn(String txt) {
        JButton b = new JButton(txt);
        b.setBackground(new Color(0x5a1010));
        b.setForeground(new Color(0xffd7d7));
        b.setFont(b.getFont().deriveFont(Font.BOLD, 16f));
        b.setFocusPainted(false);
        b.setBorder(BorderFactory.createLineBorder(new Color(0xa11), 2));
        b.addActionListener(e -> R.scram());
        return b;
    }

    private void loadScenario(String k) {
        R.loadScenario(k);
        overlayShown = false;
    }

    private void setBig(int i, String v, Color c) { bigVal[i].setText(v); bigVal[i].setForeground(c); }
    private Color col(double v, double ok, double warn) { return v > warn ? RED : (v < ok ? AMBER : GREEN); }

    private void updateAlarms() {
        if (alarmModel.size() != R.alarms.size()) {
            alarmModel.clear();
            for (Reactor.Alarm a : R.alarms.values()) alarmModel.addElement(a);
        }
        alarmList.repaint();
    }

    private void updateLog() {
        int from = logKeys.size();
        List<Reactor.LogEntry> lg = R.log;
        if (from > lg.size()) from = 0;
        for (int i = from; i < lg.size(); i++) {
            Reactor.LogEntry e = lg.get(i);
            appendLog(fmtTime(e.t) + "  " + e.msg, e.cls);
        }
        logKeys.clear();
        for (Reactor.LogEntry e : lg) logKeys.add(e.msg);
    }

    private void appendLog(String line, String cls) {
        StyledDocument doc = logPane.getStyledDocument();
        try {
            SimpleAttributeSet a = new SimpleAttributeSet();
            StyleConstants.setFontFamily(a, "Consolas");
            StyleConstants.setFontSize(a, 11);
            switch (cls == null ? "info" : cls) {
                case "crit": StyleConstants.setForeground(a, RED); StyleConstants.setBold(a, true); break;
                case "warn": StyleConstants.setForeground(a, AMBER); break;
                case "ok":   StyleConstants.setForeground(a, GREEN); break;
                default:      StyleConstants.setForeground(a, new Color(0x9fb4c6)); break;
            }
            doc.insertString(doc.getLength(), line + "\n", a);
            while (doc.getLength() > 8000) doc.remove(0, doc.getLength() - 8000);
            logPane.setCaretPosition(doc.getLength());
        } catch (Exception ignored) {}
    }

    private String fmtTime(double t) {
        long s = (long) t;
        return String.format("%02d:%02d:%02d", s / 3600, s % 3600 / 60, s % 60);
    }

    // ================= КЛАВИАТУРА =================
    private void bindKeys(JComponent c) {
        InputMap im = c.getInputMap(JComponent.WHEN_IN_FOCUSED_WINDOW);
        ActionMap am = c.getActionMap();
        im.put(KeyStroke.getKeyStroke("pressed UP"), "up"); im.put(KeyStroke.getKeyStroke("released UP"), "upR");
        im.put(KeyStroke.getKeyStroke("pressed DOWN"), "dn"); im.put(KeyStroke.getKeyStroke("released DOWN"), "dnR");
        im.put(KeyStroke.getKeyStroke("W"), "up"); im.put(KeyStroke.getKeyStroke("released W"), "upR");
        im.put(KeyStroke.getKeyStroke("S"), "dn"); im.put(KeyStroke.getKeyStroke("released S"), "dnR");
        im.put(KeyStroke.getKeyStroke("SPACE"), "stop");
        im.put(KeyStroke.getKeyStroke("A"), "az");
        am.put("up", rod(1)); am.put("upR", rod(0));
        am.put("dn", rod(-1)); am.put("dnR", rod(0));
        am.put("stop", rod(0));
        am.put("az", new AbstractAction() { public void actionPerformed(ActionEvent e) { R.scram(); } });
    }
    private AbstractAction rod(int d) {
        return new AbstractAction() { public void actionPerformed(ActionEvent e) { R.rodDir = d; } };
    }

    private static JLabel lbl(String s, int size, Color c, boolean bold) {
        JLabel l = new JLabel(s);
        l.setFont(new Font("Segoe UI", bold ? Font.BOLD : Font.PLAIN, size));
        l.setForeground(c);
        return l;
    }
    private static void styleBtn(AbstractButton b) {
        b.setFocusPainted(false);
        b.setBackground(new Color(0x18232e));
        b.setForeground(TXT);
        b.setBorder(BorderFactory.createLineBorder(LINE));
        b.setFont(b.getFont().deriveFont(11f));
    }

    // ================= РЕАЛЬНЫЙ ПЕРЕКЛЮЧАТЕЛЬ (рокер с лампой) =================
    /** Переключатель как на БЩУ: имя, рокер ВКЛ/ОТКЛ, лампа состояния. Клик — переключение. */
    class ToggleSwitch extends JPanel {
        final String name;
        boolean on;
        Runnable onChange;

        ToggleSwitch(String name, boolean init, Runnable onChange) {
            this.name = name; this.on = init; this.onChange = onChange;
            setOpaque(false);
            setPreferredSize(new Dimension(78, 46));
            setToolTipText(name + ": " + (on ? "ВКЛ" : "ОТКЛ"));
            addMouseListener(new MouseAdapter() {
                public void mousePressed(MouseEvent e) { toggle(); }
            });
        }
        void setOn(boolean v, boolean fire) {
            if (on == v) return;
            on = v;
            if (fire && onChange != null) onChange.run();
            repaint();
        }
        void toggle() { setOn(!on, true); }
        protected void paintComponent(Graphics gr) {
            super.paintComponent(gr);
            Graphics2D g = (Graphics2D) gr;
            g.setRenderingHint(RenderingHints.KEY_ANTIALIASING, RenderingHints.VALUE_ANTIALIAS_ON);
            int w = getWidth();
            // имя
            g.setFont(new Font("Segoe UI", Font.PLAIN, 10));
            g.setColor(on ? TXT : DIM);
            g.drawString(name, 0, 11);
            // дорожка-рокер
            int tw = 46, th = 15, ty = 16;
            g.setColor(new Color(0x141d26));
            g.fillRoundRect(0, ty, tw, th, 8, 8);
            g.setColor(LINE);
            g.drawRoundRect(0, ty, tw, th, 8, 8);
            // ручка
            int kx = on ? tw - 15 : 1;
            g.setColor(on ? new Color(0x2f6fbf) : new Color(0x59616b));
            g.fillRoundRect(kx, ty - 3, 14, th + 6, 7, 7);
            g.setColor(new Color(0x9fb4c6));
            g.drawRoundRect(kx, ty - 3, 14, th + 6, 7, 7);
            // подпись ВКЛ/ОТКЛ
            g.setFont(new Font("Segoe UI", Font.BOLD, 8));
            g.setColor(on ? GREEN : DIM);
            g.drawString(on ? "ВКЛ" : "ОТКЛ", 0, ty + th + 11);
            // лампа состояния
            g.setColor(on ? GREEN : new Color(0x4a2f2f));
            g.fillOval(w - 14, ty - 1, 11, 11);
            g.setColor(on ? new Color(0x7dffb0) : new Color(0x8a4a4a));
            g.drawOval(w - 14, ty - 1, 11, 11);
        }
    }

    public static void main(String[] args) {
        SwingUtilities.invokeLater(NPPGui::new);
    }
}

