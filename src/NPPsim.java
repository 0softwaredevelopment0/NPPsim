import javax.swing.SwingUtilities;

/** NPP-SIM — портативный симулятор управления АЭС (ВВЭР-1000). */
public class NPPsim {
    public static void main(String[] args) {
        if (args.length > 0 && (args[0].equals("--version") || args[0].equals("-v"))) {
            System.out.println("NPP-SIM 1.0 — симулятор управления АЭС (Java/Swing, портативная версия)");
            return;
        }
        if (args.length > 0 && args[0].equals("--selftest")) {
            PhysicsTest.main(new String[0]);
            return;
        }
        SwingUtilities.invokeLater(NPPGui::new);
    }
}
