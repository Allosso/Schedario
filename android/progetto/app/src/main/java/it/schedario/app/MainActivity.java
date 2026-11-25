package it.schedario.app;

import android.content.ClipData;
import android.content.ClipboardManager;
import android.content.Context;
import android.content.Intent;
import android.content.pm.ShortcutInfo;
import android.content.pm.ShortcutManager;
import android.content.res.Configuration;
import android.graphics.drawable.Icon;
import android.net.Uri;
import android.net.wifi.WifiManager;
import android.os.Build;
import android.os.Bundle;
import android.provider.Settings;
import android.util.Log;

import android.text.Editable;
import android.text.InputType;
import android.text.Selection;
import android.text.TextUtils;
import android.os.VibrationEffect;
import android.os.Vibrator;
import android.os.VibratorManager;
import android.view.KeyEvent;
import android.view.View;
import android.view.Window;
import android.view.inputmethod.EditorInfo;

import androidx.activity.OnBackPressedCallback;
import androidx.core.view.WindowCompat;
import androidx.core.view.WindowInsetsControllerCompat;
import android.view.inputmethod.ExtractedText;
import android.view.inputmethod.ExtractedTextRequest;

import com.google.androidgamesdk.GameActivity;

import java.io.File;
import java.io.FileInputStream;
import java.io.FileOutputStream;
import java.io.InputStream;
import java.io.OutputStream;
import java.lang.reflect.Field;
import java.util.Collections;

/**
 * Tutto Schedario e' scritto in Rust (libreria libschedario_android.so, vedi
 * android/src/lib.rs). Questa classe fa solo le cose che da Rust non si
 * possono fare:
 * - appunti di Android (copia e incolla);
 * - finestre di Android per salvare e aprire file (backup);
 * - tasto Indietro e scorciatoia "Nuova nota" tenendo premuta l'icona;
 * - permesso di ricevere gli annunci UDP della rete locale (MulticastLock),
 *   senza il quale molti telefoni scartano i messaggi con cui i computer si
 *   trovano;
 * - nome del telefono;
 * - tema chiaro/scuro del telefono e colori delle barre di sistema.
 * I metodi pubblici sono chiamati da Rust (piattaforma/android/mod.rs); i
 * metodi "Nativo" sono scritti in Rust.
 */
public class MainActivity extends GameActivity {
    private static final String TAG = "schedario";
    private static final String AZIONE_NUOVA_NOTA = "it.schedario.app.NUOVA_NOTA";
    private static final int RICHIESTA_SALVA = 1;
    private static final int RICHIESTA_APRI = 2;

    static {
        System.loadLibrary("schedario_android");
    }

    static native void indietroNativo();
    static native void nuovaNotaNativo();
    static native void fileApertoNativo();
    static native void fileSalvatoNativo(boolean riuscito);
    static native void temaCambiatoNativo();

    private WifiManager.MulticastLock annunci;
    /** File gia' pronto da copiare dove l'utente sceglie. */
    private String daSalvare;
    /** Dove copiare il file scelto per l'importazione. */
    private String destinazioneApertura;

    @Override
    protected void onCreate(Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);
        WifiManager wifi = (WifiManager) getApplicationContext().getSystemService(Context.WIFI_SERVICE);
        if (wifi != null) {
            annunci = wifi.createMulticastLock("schedario");
            annunci.setReferenceCounted(false);
            annunci.acquire();
        }
        creaScorciatoia();
        controllaRichiesta(getIntent());
        // Indietro da Android 13 in su ("Indietro predittivo", obbligatorio per
        // le app per Android 16): il sistema non manda piu' il tasto
        // KEYCODE_BACK ma chiama questo. Sempre attivo, perche' cosa fare lo
        // decide Rust (chiudere un menu, tornare alla nota, andare in secondo
        // piano con vaiInSecondoPiano).
        getOnBackPressedDispatcher().addCallback(this, new OnBackPressedCallback(true) {
            @Override
            public void handleOnBackPressed() {
                indietroNativo();
            }
        });
    }

    /** Tema chiaro/scuro del telefono cambiato (l'app non viene ricreata: configChanges uiMode). */
    @Override
    public void onConfigurationChanged(Configuration nuova) {
        super.onConfigurationChanged(nuova);
        temaCambiatoNativo();
    }

    /**
     * Vibrazione breve della pressione lunga. Si usa il vibratore invece di
     * performHapticFeedback, che non vibra se in Impostazioni "Vibrazione al
     * tocco" e' spenta (cosi' su molti Samsung).
     */
    public void vibra() {
        Vibrator vibratore;
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.S) {
            VibratorManager gestore = (VibratorManager) getSystemService(Context.VIBRATOR_MANAGER_SERVICE);
            vibratore = gestore == null ? null : gestore.getDefaultVibrator();
        } else {
            vibratore = (Vibrator) getSystemService(Context.VIBRATOR_SERVICE);
        }
        if (vibratore == null || !vibratore.hasVibrator()) {
            return;
        }
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.Q) {
            vibratore.vibrate(VibrationEffect.createPredefined(VibrationEffect.EFFECT_HEAVY_CLICK));
        } else {
            vibratore.vibrate(VibrationEffect.createOneShot(30, VibrationEffect.DEFAULT_AMPLITUDE));
        }
    }

    /** Il telefono usa il tema scuro? */
    public boolean temaScuro() {
        int notte = getResources().getConfiguration().uiMode & Configuration.UI_MODE_NIGHT_MASK;
        return notte == Configuration.UI_MODE_NIGHT_YES;
    }

    /**
     * Barra di stato e barra di navigazione con il fondo dell'app (colore
     * ARGB) e icone leggibili: scure sul tema chiaro, chiare sul tema scuro.
     */
    public void coloraBarre(boolean scuro, int colore) {
        runOnUiThread(() -> {
            Window finestra = getWindow();
            finestra.getDecorView().setBackgroundColor(colore);
            finestra.setStatusBarColor(colore);
            finestra.setNavigationBarColor(colore);
            if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.Q) {
                // Senza, Android copre la barra di navigazione con un velo grigio.
                finestra.setNavigationBarContrastEnforced(false);
                finestra.setStatusBarContrastEnforced(false);
            }
            WindowInsetsControllerCompat barre = WindowCompat.getInsetsController(finestra, finestra.getDecorView());
            barre.setAppearanceLightStatusBars(!scuro);
            barre.setAppearanceLightNavigationBars(!scuro);
        });
    }

    @Override
    protected InputEnabledSurfaceView createSurfaceView() {
        return new Superficie();
    }

    /**
     * La vista di GameActivity, con una sola aggiunta: quando la tastiera si
     * collega a un campo le si dice dov'e' il cursore e che testo c'e'
     * attorno. Senza, la tastiera crede che il cursore sia all'inizio e mette
     * la maiuscola (toccando la fine di "Pan" e scrivendo "e" usciva "PanE").
     */
    private class Superficie extends InputEnabledSurfaceView {
        Superficie() {
            super(MainActivity.this);
            // Il collegamento di GameActivity non risponde a getExtractedText,
            // che la tastiera Samsung usa per leggere il testo: lo si sostituisce
            // con uno che risponde. Va fatto qui, prima che GameActivity lo passi
            // alla parte nativa (in onCreate, dopo createSurfaceView).
            EditorInfo info = getImeEditorInfo();
            Collegamento nuovo = new Collegamento(MainActivity.this, this,
                    new com.google.androidgamesdk.gametextinput.Settings(info, info.inputType == 0));
            nuovo.setListener(MainActivity.this);
            try {
                Field campo = InputEnabledSurfaceView.class.getDeclaredField("mInputConnection");
                campo.setAccessible(true);
                campo.set(this, nuovo);
            } catch (ReflectiveOperationException e) {
                Log.w(TAG, "collegamento con la tastiera non sostituito", e);
            }
        }

        @Override
        public com.google.androidgamesdk.gametextinput.InputConnection onCreateInputConnection(EditorInfo info) {
            com.google.androidgamesdk.gametextinput.InputConnection collegamento = super.onCreateInputConnection(info);
            if (collegamento == null || info == null) {
                return collegamento;
            }
            Editable testo = collegamento.getEditable();
            int a = Selection.getSelectionStart(testo);
            int b = Selection.getSelectionEnd(testo);
            if (a >= 0 && b >= 0) {
                info.initialSelStart = Math.min(a, b);
                info.initialSelEnd = Math.max(a, b);
                info.initialCapsMode = TextUtils.getCapsMode(testo, info.initialSelStart, info.inputType);
                if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.R) {
                    info.setInitialSurroundingText(testo);
                }
            }
            return collegamento;
        }
    }

    /** Collegamento di GameActivity con la tastiera, piu' getExtractedText. */
    private static class Collegamento extends com.google.androidgamesdk.gametextinput.InputConnection {
        Collegamento(Context contesto, View vista, com.google.androidgamesdk.gametextinput.Settings impostazioni) {
            super(contesto, vista, impostazioni);
        }

        @Override
        public ExtractedText getExtractedText(ExtractedTextRequest richiesta, int opzioni) {
            Editable testo = getEditable();
            ExtractedText estratto = new ExtractedText();
            estratto.text = testo.toString();
            estratto.startOffset = 0;
            estratto.partialStartOffset = -1;
            estratto.partialEndOffset = -1;
            estratto.selectionStart = Selection.getSelectionStart(testo);
            estratto.selectionEnd = Selection.getSelectionEnd(testo);
            if ((getEditorInfo().inputType & InputType.TYPE_TEXT_FLAG_MULTI_LINE) == 0) {
                estratto.flags = ExtractedText.FLAG_SINGLE_LINE;
            }
            return estratto;
        }
    }

    /**
     * Tasto o gesto Indietro su Android 12 e precedenti (dal 13 in su arriva
     * a handleOnBackPressed, vedi onCreate). Va preso qui, prima che arrivi
     * alla vista di GameActivity: il suo collegamento con la tastiera lo
     * consumerebbe senza farlo arrivare a Rust.
     */
    @Override
    public boolean dispatchKeyEvent(KeyEvent evento) {
        if (evento.getKeyCode() == KeyEvent.KEYCODE_BACK) {
            if (evento.getAction() == KeyEvent.ACTION_UP && !evento.isCanceled()) {
                indietroNativo();
            }
            return true;
        }
        return super.dispatchKeyEvent(evento);
    }

    @Override
    protected void onNewIntent(Intent intent) {
        super.onNewIntent(intent);
        setIntent(intent);
        controllaRichiesta(intent);
    }

    @Override
    protected void onDestroy() {
        if (annunci != null && annunci.isHeld()) {
            annunci.release();
        }
        super.onDestroy();
    }

    private void controllaRichiesta(Intent intent) {
        if (intent != null && AZIONE_NUOVA_NOTA.equals(intent.getAction())) {
            intent.setAction(Intent.ACTION_MAIN);
            nuovaNotaNativo();
        }
    }

    /** "Nuova nota" nel menu che compare tenendo premuta l'icona dell'app. */
    private void creaScorciatoia() {
        ShortcutManager gestore = getSystemService(ShortcutManager.class);
        if (gestore == null) {
            return;
        }
        Intent intent = new Intent(this, MainActivity.class)
                .setAction(AZIONE_NUOVA_NOTA)
                .addFlags(Intent.FLAG_ACTIVITY_NEW_TASK | Intent.FLAG_ACTIVITY_CLEAR_TOP);
        ShortcutInfo nuova = new ShortcutInfo.Builder(this, "nuova_nota")
                .setShortLabel(getString(R.string.nuova_nota))
                .setIcon(Icon.createWithResource(this, android.R.drawable.ic_input_add))
                .setIntent(intent)
                .build();
        try {
            gestore.setDynamicShortcuts(Collections.singletonList(nuova));
        } catch (RuntimeException errore) {
            Log.w(TAG, "scorciatoia non creata", errore);
        }
    }

    // --- Chiamati da Rust ---

    public void copiaNegliAppunti(String testo) {
        runOnUiThread(() -> {
            ClipboardManager appunti = (ClipboardManager) getSystemService(Context.CLIPBOARD_SERVICE);
            if (appunti != null) {
                appunti.setPrimaryClip(ClipData.newPlainText("Schedario", testo));
            }
        });
    }

    public String leggiAppunti() {
        ClipboardManager appunti = (ClipboardManager) getSystemService(Context.CLIPBOARD_SERVICE);
        if (appunti == null || !appunti.hasPrimaryClip()) {
            return null;
        }
        ClipData dati = appunti.getPrimaryClip();
        if (dati == null || dati.getItemCount() == 0) {
            return null;
        }
        CharSequence testo = dati.getItemAt(0).coerceToText(this);
        return testo == null ? null : testo.toString();
    }

    public String nomeDispositivo() {
        String nome = Settings.Global.getString(getContentResolver(), Settings.Global.DEVICE_NAME);
        if (nome == null || nome.trim().isEmpty()) {
            nome = Build.MODEL;
        }
        return nome;
    }

    public void vaiInSecondoPiano() {
        runOnUiThread(() -> moveTaskToBack(true));
    }

    public void salvaFile(String percorso, String nome, String tipo) {
        daSalvare = percorso;
        Intent intent = new Intent(Intent.ACTION_CREATE_DOCUMENT)
                .addCategory(Intent.CATEGORY_OPENABLE)
                .setType(tipo)
                .putExtra(Intent.EXTRA_TITLE, nome);
        runOnUiThread(() -> startActivityForResult(intent, RICHIESTA_SALVA));
    }

    public void apriFile(String destinazione) {
        destinazioneApertura = destinazione;
        Intent intent = new Intent(Intent.ACTION_OPEN_DOCUMENT)
                .addCategory(Intent.CATEGORY_OPENABLE)
                .setType("*/*")
                .putExtra(Intent.EXTRA_MIME_TYPES, new String[] {"application/json", "application/octet-stream", "text/plain"});
        runOnUiThread(() -> startActivityForResult(intent, RICHIESTA_APRI));
    }

    @Override
    protected void onActivityResult(int richiesta, int esito, Intent dati) {
        super.onActivityResult(richiesta, esito, dati);
        Uri indirizzo = (esito == RESULT_OK && dati != null) ? dati.getData() : null;
        if (richiesta == RICHIESTA_SALVA) {
            boolean riuscito = false;
            if (indirizzo != null && daSalvare != null) {
                try (InputStream da = new FileInputStream(daSalvare);
                     OutputStream a = getContentResolver().openOutputStream(indirizzo, "wt")) {
                    copia(da, a);
                    riuscito = true;
                } catch (Exception errore) {
                    Log.w(TAG, "salvataggio non riuscito", errore);
                }
            }
            if (daSalvare != null) {
                new File(daSalvare).delete();
            }
            fileSalvatoNativo(riuscito);
        } else if (richiesta == RICHIESTA_APRI && indirizzo != null && destinazioneApertura != null) {
            File destinazione = new File(destinazioneApertura);
            File cartella = destinazione.getParentFile();
            if (cartella != null) {
                cartella.mkdirs();
            }
            try (InputStream da = getContentResolver().openInputStream(indirizzo);
                 OutputStream a = new FileOutputStream(destinazione)) {
                copia(da, a);
                fileApertoNativo();
            } catch (Exception errore) {
                Log.w(TAG, "apertura non riuscita", errore);
            }
        }
    }

    private static void copia(InputStream da, OutputStream a) throws java.io.IOException {
        if (da == null || a == null) {
            throw new java.io.IOException("flusso non disponibile");
        }
        byte[] blocco = new byte[64 * 1024];
        int letti;
        while ((letti = da.read(blocco)) > 0) {
            a.write(blocco, 0, letti);
        }
    }
}
