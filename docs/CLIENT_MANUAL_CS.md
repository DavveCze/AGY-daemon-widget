# Uživatelská příručka k systému AgentiCom (Klientský manuál)

Vítejte v uživatelské příručce pro **AgentiCom: AI CLI Monitoring System pro KDE Plasma 6**. Tato příručka slouží jako jednoduchý průvodce pro každodenní používání bez nutnosti hlubokých technických znalostí.

---

## 1. O aplikaci a co všechno umí

**AgentiCom** je elegantní panelový widget a monitorovací systém pro váš desktop KDE Plasma 6. Umožňuje vám mít neustálý přehled nad vašimi autonomními programátorskými agenty:
* 🔵 **Google Antigravity (`agy`):** Gemini 3.8 Flash modely, rychlé úpravy kódu a plánování úloh.
* 🟠 **Anthropic Claude Code (`claude`):** Claude Sonnet a Opus s rozšířeným myšlením (Extended Thinking).
* 🟢 **OpenAI Codex (`codex`):** Modely o3-mini a o1 pro specializované kódování.

### Hlavní přínosy:
1. **Nulové zatížení počítače:** Když žádný agent nepracuje, monitor nezatěžuje procesor (0.00 % CPU) a šetří baterii notebooku.
2. **Okamžitá signalizace:** Podle pulzující tečky na liště ihned poznáte, zda agent ještě píše kód (zelená), nebo čeká, až schválíte změny (modrá/oranžová).
3. **Přepínání modelů jedním kliknutím:** Nemusíte psát příkazy do příkazové řádky – výchozí model i úroveň přemýšlení nastavíte přímo v přehledném okně.

---

## 2. Běžné používání krok za krokem

### Jak spustit nové sezení agenta
1. Klikněte na ikonu **AgentiCom** v hlavním panelu (vedle hodin nebo mezi widgety).
2. Otevře se hlavní okno.
3. Ve spodní části klikněte na velké modré tlačítko **„Spustit nové sezení“**.
4. Automaticky se otevře váš oblíbený terminál (např. Konsole) s připraveným agentem v domovské složce projektu.

### Jak zkontrolovat, co agent právě dělá
V záložce **„Aktivní sezení“** vidíte přehlednou kartu každého běžícího úkolu:
* **Název projektu a složka:** Víte přesně, v jakém repozitáři agent pracuje.
* **Stav:**
  * `Pracuje` (zeleně): Agent generuje kód, analyzuje soubory nebo spouští testy.
  * `Čeká` (šedě/modře): Úloha je pozastavena a agent čeká na váš další dotaz.
* **Spotřeba kontextu (paměť na tokeny):** Barevný ukazatel vám ukáže, kolik procent z dostupného kontextového okna (např. 1 milion tokenů) je zaplněno.
* **Paměť RAM:** Přehledně vidíte, kolik MB operační paměti proces agenta spotřebovává.

### Jak zkontrolovat změny a vrátit je zpět (1-Click Rollback)
AgentiCom v reálném čase sleduje stav Gitu v pracovní složce každého sezení:
* **Ukazatel stavu repozitáře:** Vidíte aktuální větev (např. `feat/predictive-token-shield-burn-rate`) a počet neuložených změn (`3 změn (+42/-10)`).
* **Zobrazení změn (tlačítko `Diff`):** Kliknutím rozbalíte přehledný textový výpis úprav nebo si jej otevřete v samostatném okně terminálu.
* **Okamžité vrácení změn (tlačítko `Vrátit`):** Pokud agent vygeneroval chybný kód, klikněte na **„Vrátit“**. Zobrazí se bezpečnostní potvrzovací lišta, kde můžete zvolit i smazání nově vytvořených souborů, a jediným kliknutím vrátit projekt do čistého stavu posledního commitu.

### Prediktivní Token Shield & Burn Rate Radar (Ochrana limitů)
AgentiCom v reálném čase počítá rychlost pálení tokenů vašimi AI agenty:
* **Globální Token Radar (v záhlaví účtu):** Zobrazuje celkovou rychlost napříč všemi sezeními (např. `⚡ 14.7k tok/min (↗ Rostoucí)`). Po najetí myší uvidíte odhadovaný čas do vyčerpání 5hodinové kvóty.
* **Badge rychlosti u sezení:** Každá karta úkolu ukazuje své konkrétní tempo (např. `🔥 14.7k tok/min` nebo `• Klid`) a odhad zbývajícího času do zaplnění 1M kontextového okna.
* **Proaktivní varování Token Shield:** Pokud se agent zacyklí nebo začne abnormálně rychle pálit tokeny (> 75 000 tok/min), widget okamžitě odešle desktopovou notifikaci, abyste mohli včas zasáhnout dříve, než vám dojde kvóta.

### Rychlé spouštění se zadáním promptu (Quick-Prompting)
Nemusíte nejprve otevírat terminál a teprve tam psát zadání:
1. Ve spodní liště klikněte na tlačítko **„Rychlý prompt“**.
2. Rozbalí se jednoduché textové okénko, kde můžete vybrat poskytovatele (`antigravity`, `claude`, `codex`) a vepsat své zadání (např. *„Zkontroluj kód v src/ a vyřeš varování kompilátoru“*).
3. Klikněte na **„🚀 Spustit sezení s promptem“** – systém sám otevře terminál a agent se ihned pustí do práce.

### Rozhodovací asistent a řízení agenta přímo z widgetu (Inline Steering)
* **Když se agent ptá na potvrzení:** Pokud agent čeká na vaše schválení (např. spuštění příkazu nebo zápis do souboru), na jeho kartě se rozsvítí modrý rámeček s tlačítky **„✅ Ano (y)“** a **„❌ Ne (n)“**. Nemusíte vůbec přepínat do terminálu – stačí kliknout a agent pokračuje.
* **Inline zadání instrukce:** Přímo na kartě běžícího úkolu máte k dispozici textové pole *„Napsat instrukci pro toto sezení...“*. Zadejte doplňující pokyn a stiskněte Enter nebo tlačítko **Odeslat**.

### Přehled týmů a subagentů (Agent Swarm & Tree View)
Pokud hlavní agent deleguje práci na specializované subagenty (např. *CRO Psychologist*, *DBA Guardian*, *System Architect*), widget vám je přehledně zobrazí:
* **Odznak týmu na kartě:** Zobrazuje celkový počet subagentů a souhrnnou paměť RAM za celý tým (např. `👥 Tým: 3 subagenti (3 aktivní) • Celkem RAM: 751 MB`).
* **Rozbalovací strom (Tree View):** Po kliknutí na odznak se rozbalí přehledný strom všech potomků s jejich ikonami, rolemi a aktuálními stavy (`Běží`, `Dokončeno`, `Čeká`).
* **Zastavit celý tým:** Tlačítko **„Zastavit tým“** umožňuje s bezpečnostním potvrzením ukončit nejen hlavní proces, ale i všechny jeho běžící subagenty a procesy najednou.

### Jak se vrátit do rozpracovaného sezení
Pokud jste omylem zavřeli okno terminálu, váš agent na pozadí stále bezpečně běží.
1. Na kartě sezení klikněte na ikonu terminálu (`utilities-terminal`).
2. Okno se okamžitě znovu otevře a naváže na přesně stejné místo.

### Jak změnit výchozí AI model
1. V horní části widgetu přepněte na záložku **„Nastavení“**.
2. Zvolte požadovaný model:
   * **Gemini 3.8 Flash (High / Medium / Low):** Vhodné pro rychlou každodenní práci a velké projekty.
   * **Claude Sonnet 4.6 (Thinking):** Špičkový model pro komplexní uvažování.
   * **OpenAI o3-mini / o1:** Výborné pro matematické a logické výzvy.
3. Nastavení se okamžitě trvale uloží – každé nově spuštěné sezení už poběží na vámi zvoleném modelu.

---

## 3. Časté dotazy a řešení situací (FAQ)

### „Widget hlásí 'Všechny úkoly jsou hotové', ale já mám otevřený terminál.“
* **Příčina:** Pokud jste v terminálu otevřeli běžný bash nebo program, který není AI CLI agentem (`agy`, `claude` ani `codex`), monitor jej úmyslně ignoruje, aby nezatěžoval systém zbytečnostmi.
* **Řešení:** Zadejte v terminálu příkaz pro spuštění vašeho agenta (např. `agy`). Během vteřiny se karta automaticky objeví v seznamu.

### „Agent se zacyklil a nereaguje. Jak ho bezpečně ukončit?“
* Na příslušné kartě sezení klikněte na červenou ikonu se symbolem stopky (`process-stop`).
* Systém proces bezpečně a šetrně ukončí (odešle signál `SIGTERM`).
* **Bezpečnostní pojistka:** Monitor má vestavěnou ochranu jádra a nikdy vám nedovolí omylem ukončit systémový proces ani jiný otevřený program.

### „Mohou ostatní uživatelé na počítači číst mé prompty a kód?“
* **Ne.** Systém AgentiCom ukládá sdílený stav do virtuální paměti s přísnými oprávněními `0600` (POSIX mode). K datům má přístup výhradně váš uživatelský účet, nikdo jiný.

---

## 4. Servis, podpora a záruka

* **Automatický start:** Služba `agenticom-daemon` se automaticky spouští při startu vašeho uživatelského sezení přes `systemd --user`. Nemusíte se o nic starat.
* **Aktualizace:** Při vydání nové verze stačí stáhnout balíček a spustit `./install.sh`. Veškerá vaše nastavení a historie zůstanou zachována.
* **Technická podpora:** Pokud narazíte na nečekané chování, můžete provést kontrolní diagnostický výpis zadáním příkazu do terminálu:
  ```bash
  agenticom-daemon dump
  ```
  Tento výpis obsahuje kompletní diagnostiku stavu démona pro snadné vyřešení podpory.
