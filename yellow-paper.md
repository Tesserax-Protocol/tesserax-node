# **THE SANCTUARY PROTOCOL**

## **A Mathematical Definition of Adaptive Scarcity & Quantum-Resistant Consensus**

**Version:** 0.1 (Draft)

**Author:** Minerva & Gemini (The Architect)  

**Abstract:** 
Dokumen ini mendefinisikan Sanctuary Protocol, sebuah state-machine terdesentralisasi yang dibangun di atas kerangka Substrate. Berbeda dengan model deflasi statis Bitcoin, Sanctuary memperkenalkan Adaptive Scarcity Mechanism (ASM) yang diatur oleh konstanta universal ($\pi, e, \phi$). Makalah ini memberikan definisi formal untuk fungsi emisi sigmoid, logika halving dinamis berdasarkan aktivitas jaringan, dan skema tanda tangan hibrida (ECDSA + Lattice-Based PQC).

---

### **TABLE OF CONTENTS**

#### **1\. Introduction: The Philosophy of Universal Constants**

* **1.1. Latar Belakang:** Keterbatasan model inflasi Ethereum dan model deflasi kaku Bitcoin.  

* **1.2. The Thesis:** Mengusulkan bahwa supply mata uang harus mencerminkan pola pertumbuhan alami (Natural Growth Patterns), bukan keputusan arbitrer manusia.

#### **2\. The Sanctuary Constant (Economic Primitives)**

* **2.1. Definisi Konstanta Universal:**  
  * Mendefinisikan $\pi$ (Siklus), $e$ (Pertumbuhan), dan $\phi$ (Proporsi) dalam konteks presisi *fixed-point arithmetic* (u128) untuk determinisme komputasi.  
* **2.2. The Maximum Supply Formula ($S_{max}$):**  
  * Pembuktian matematis batas asimtotik supply.  
      $$S_{max} = \lfloor \pi \times e \times \phi \times 10^6 \rfloor \approx 13,817,422$$
* **2.3. Kurva Emisi Sigmoid:**  
  * Menjabarkan fungsi $S(t)$ yang menggantikan emisi linear.  
    $$S(t) = S_{max} \cdot \frac{1}{1 + e^{-k(t - t_0)}}$$
  * *Simulasi on-chain:* Bagaimana fungsi ini diimplementasikan dalam Rust tanpa *floating point* (menggunakan *Lookup Tables* atau deret Taylor).

#### **3\. Adaptive Scarcity Mechanism (ASM)**

* **3.1. Network Activity Quotient ($Q_{net}$):**  
  * Cara protokol menghitung "kesehatan jaringan" secara *trustless*.  
  * Metode memfilter *spam addresses* untuk mendapatkan jumlah *Active Addresses* yang valid.  
* **3.2. Dynamic Halving Algorithm:**  
  * Rumus inti yang mengubah Block Reward ($R_b$) berdasarkan $Q_{net}$.  
    $$R_b(t, Q_{net}) = \frac{R_{base}}{2^{\mathcal{H}(t) \cdot \sqrt{Q_{net}}}}$$
  * Pembuktian bahwa peningkatan adopsi mempercepat kelangkaan (Scarcity Acceleration).

#### **4\. Cryptographic Architecture (Identity & Security)**

* **4.1. Hybrid Signature Scheme:**  
  * Struktur data *Account ID* di Sanctuary.  
  * Mendukung kurva secp256k1 (Ethereum compatibility) dan skema PQC (Crystal-Dilithium / Falcon).  
* **4.2. Post-Quantum State Transition:**  
  * Proses *key-rotation* dari Legacy Address ke Quantum Address tanpa kehilangan aset.  
* **4.3. The "Sentinel" Module:**  
  * Spesifikasi teknis *runtime module* yang menolak transaksi jika terdeteksi pola serangan Shor (Quantum decryption attempt).

#### **5\. Consensus & Finality (The Engine)**

* **5.1. NPoS (Nominated Proof of Stake):**  
  * Menggunakan algoritma Phragmén untuk distribusi validator yang optimal dan adil.  
* **5.2. Block Production (BABE):**  
  * *Blind Assignment for Blockchain Extension* \- mekanisme probabilistik pembuatan blok.  
* **5.3. Finality Gadget (GRANDPA):**  
  * Mekanisme finalitas deterministik yang memisahkan pembuatan blok dari konfirmasi blok.

#### **6\. The Virtual Machine (Execution Layer)**

* **6.1. Frontier Integration:**  
  * Bagaimana EVM (Ethereum Virtual Machine) berjalan sebagai *guest* di dalam *host* Substrate.  
* **6.2. Precompiles for Mathematics:**  
  * Membuat *custom precompiled contracts* agar dApps Solidity bisa mengakses data *Sanctuary Constant* dan status PQC dengan biaya gas murah.

---

# 

# **CHAPTER 2: THE SANCTUARY CONSTANT (ECONOMIC PRIMITIVES)**

## **2.1. Definisi Konstanta Universal**

Landasan ekonomi Sanctuary tidak dibangun di atas keputusan komite terpusat, melainkan diturunkan dari konstanta irasional yang mengatur fenomena alam semesta: siklus ($\pi$), pertumbuhan ($e$), dan proporsi ($\phi$).

Dalam konteks Sanctuary Protocol, konstanta ini didefinisikan sebagai nilai tetap (*hard-coded constants*) dalam presisi tinggi untuk menghindari ambiguitas *floating-point* pada lapisan konsensus.

### **Definisi 2.1.1 (Konstanta Basis)**

Kami mendefinisikan himpunan konstanta $\mathbb{C}$ dalam ruang bilangan real $\mathbb{R}$:

$$\pi \approx 3.1415926535... \quad (\text{Archimedes' constant})$$
$$e \approx 2.7182818284... \quad (\text{Euler's number})$$
$$\phi \approx 1.6180339887... \quad (\text{The Golden Ratio})$$

### **Definisi 2.1.2 (Representasi Fixed-Point Blockchain)**

Karena *Substrate Runtime* (dan blockchain pada umumnya) tidak mendukung operasi *floating-point* demi determinisme, kami merepresentasikan konstanta ini dalam format **u128** dengan presisi 9 desimal ($10^9$) sebagai basis perhitungan internal sebelum penskalaan token ($10^{18}$).

Dalam implementasi **Rust**, ini didefinisikan dalam modul Traits:


```rust
pub trait UniversalConstants {
    const PI: u128  = 3_141_592_653; // 3.141592653 * 10^9
    const E: u128   = 2_718_281_828; // 2.718281828 * 10^9
    const PHI: u128 = 1_618_033_988; // 1.618033988 * 10^9
}
```
---

## **2.2. The Maximum Supply Formula ($S_{max}$)**

Supply maksimum ($S_{max}$) dari token $\$SANC$ bukanlah batas yang dicapai secara tiba-tiba (seperti *hard cap* Bitcoin yang berhenti mendadak), melainkan sebuah asimtot matematis yang didekati seiring waktu $t \to \infty$.

### **Proposisi 2.2 (The Sanctuary Constant)**

Supply maksimum didefinisikan sebagai hasil perkalian dari ketiga konstanta basis, diskalakan dengan faktor $10^6$ (satu juta).

$$S_{max} = \lfloor \pi \times e \times \phi \times 10^6 \rfloor$$
Perhitungan nilai eksak:

1. $\pi \times e \approx 8.539734222...$
2. $(\pi \times e) \times \phi \approx 13.817422188...$
3. $S_{max} = \lfloor 13.817422188... \times 10^6 \rfloor = 13,817,422$

Maka, batas suplai protokol ditetapkan secara permanen pada angka **13,817,422 SANC**.

**Signifikansi Teoretis:** Angka ini merepresentasikan "Volume Ideal" dari sebuah balok teoretis dengan panjang sisi $\pi$, $e$, dan $\phi$. Ini menyimbolkan stabilitas tiga dimensi yang dibangun dari konstanta alam.

---

## **2.3. Kurva Emisi Sigmoid (The Breathing Scarcity)**

Berbeda dengan fungsi tangga (*step function*) pada Bitcoin yang memotong reward setiap 4 tahun secara kasar, Sanctuary menggunakan **Fungsi Logistik (Sigmoid Curve)** untuk emisi token. Ini menciptakan distribusi yang organik: lambat di awal (genesis), akselerasi cepat (fase adopsi), dan deselerasi alami (fase maturitas).

![Image of Sigmoid function graph][image1]


### **Definisi 2.3.1 (Fungsi Pasokan Kumulatif)**

Total pasokan yang beredar pada waktu $t$ (dimana $t$ adalah *block number*) ditentukan oleh fungsi:

$$S(t) = \frac{S_{max}}{1 + e^{-k(t - t_0)}}$$
Dimana:

* $S_{max} = 13,817,422 \times 10^{18}$ (dalam unit terkecil).
* $k$ adalah *Growth Rate Constant* (laju adopsi).
* $t_0$ adalah *Inflection Point* (titik tengah waktu emisi, di mana 50% supply telah tercetak).
* $e$ adalah basis logaritma natural.

### **Definisi 2.3.2 (Fungsi Reward Per Blok)**

Reward per blok ($R_b$) adalah turunan pertama dari fungsi pasokan kumulatif $S(t)$ terhadap waktu $t$.$$R_b(t) = \frac{dS(t)}{dt} = k \cdot S(t) \cdot \left(1 - \frac{S(t)}{S_{max}}\right)$$Ini menghasilkan kurva distribusi berbentuk lonceng (Bell Curve). Reward dimulai dari rendah, memuncak di pertengahan fase adopsi, lalu menurun secara asimtotik menuju nol.

---

## **2.4. Implementasi Deterministik di Substrate (Rust)**

Tantangan utama dalam menerapkan persamaan 2.3.1 di blockchain adalah penghitungan eksponensial natural ($e^x$) tanpa *floating point*.

Untuk menyelesaikan ini di **Substrate Runtime**, kita menggunakan pendekatan **Deret Taylor (Taylor Series Approximation)** atau **Padé Approximant** yang diimplementasikan menggunakan pustaka sp\_arithmetic (Fixed Point Math).

### **Algoritma 2.4 (Aproksimasi Eksponensial On-Chain)**

Untuk menghitung $e^x$ dimana $x$ adalah bilangan fixed-point negatif (dari persamaan $-k(t-t\_0)$):

$$e^x \approx 1 + x + \frac{x^2}{2!} + \frac{x^3}{3!} + \dots + \frac{x^n}{n!}$$
Dalam kode Rust untuk modul pallet-tokenomics:

```Rust

use sp\_runtime::traits::{Saturating, CheckedDiv};  
use sp\_arithmetic::{FixedU128, FixedPointNumber};

// Menghitung S(t) menggunakan logistic function approximation  
fn calculate\_current\_supply(  
    current\_block: u64,   
    inflection\_point: u64,   
    growth\_rate: FixedU128  
) \-\> u128 {  
    let s\_max \= 13\_817\_422\_000\_000\_000\_000\_000\_000u128; // 13.8M \* 10^18  
      
    // Konversi block delta ke fixed point  
    let t\_delta \= if current\_block \> inflection\_point {  
        FixedU128::from(current\_block \- inflection\_point)  
    } else {  
        // Handle fase awal (pre-inflection)  
        // Logika inverted untuk eksponen positif  
        return calculate\_pre\_inflection\_supply(...)  
    };

    // Eksponen: \-k \* (t \- t0)  
    let exponent \= growth\_rate.saturating\_mul(t\_delta);  
      
    // Hitung penyebut: 1 \+ e^(-exponent)  
    // Menggunakan fungsi exp() custom untuk FixedU128  
    let e\_pow \= fixed\_exp(exponent);   
    let denominator \= FixedU128::from(1).saturating\_add(e\_pow);  
      
    // Hasil: S\_max / denominator  
    let supply\_fixed \= FixedU128::from\_inner(s\_max)  
        .checked\_div(\&denominator)  
        .unwrap\_or(FixedU128::from(0));  
          
    supply\_fixed.into\_inner()  
}
```
### **Implikasi Keamanan Ekonomi**

Penggunaan kurva matematika yang mulus (smooth curve) ini mencegah serangan *reward shock* yang sering terjadi pada koin PoW saat halving terjadi, di mana penambang sering mematikan mesin massal karena profitabilitas anjlok tiba-tiba. Di Sanctuary, penurunan reward terjadi sangat gradual di setiap blok, memberikan stabilitas ekonomi bagi validator.

---

# **CHAPTER 3: ADAPTIVE SCARCITY MECHANISM (ASM)**

## **3.1. Filosofi: Relativitas Waktu Ekonomi**

Dalam fisika, waktu bersifat relatif terhadap gravitasi (Dilatasi Waktu). Dalam Sanctuary Protocol, kami menerapkan prinsip serupa: **Waktu Ekonomi relatif terhadap Aktivitas Jaringan.**

Pada model Bitcoin standar, halving terjadi setiap 210.000 blok terlepas dari apakah jaringan sedang mati suri atau meledak. Ini tidak efisien. Sanctuary memperkenalkan konsep **Time Dilation** (Dilatasi Waktu) pada jadwal emisi.

* **Jaringan Sepi (Low Entropy):** Waktu berjalan normal. Emisi lambat untuk menjaga insentif validator jangka panjang.  
* **Jaringan Padat (High Entropy):** Waktu dipercepat. Emisi berkurang lebih cepat (halving dipercepat) untuk merespons permintaan tinggi dengan kelangkaan yang lebih tinggi (*Supply Shock*).

---

## **3.2. Network Activity Quotient ($Q\_{net}$)**

Untuk mengukur "gravitasi" jaringan, kita mendefinisikan $Q\_{net}$ (Quotient of Network Activity). Ukuran ini harus resisten terhadap manipulasi (*Sybil-resistant*).

### **Definisi 3.2.1 (Metrik Aktivitas)**

$Q\_{net}$ pada epoch $i$ dihitung berdasarkan rata-rata tertimbang dari tiga variabel:

$$Q_{net}^{(i)} = w_1 \cdot \ln(1 + V_{active}) + w_2 \cdot \ln(1 + T_{vol}) + w_3 \cdot \frac{B_{burned}}{S_{circ}}$$
Dimana:

* $V\_{active}$: Jumlah validator aktif dan nominator unik (Staking participation).  
* $T\_{vol}$: Volume transaksi on-chain dalam $SANC$ (mencegah spam transaksi 0 value).  
* $B\_{burned}$: Jumlah fee yang dibakar (indikator kemacetan jaringan/demand blockspace).  
* $S\_{circ}$: Supply yang beredar.  
* $w\_1, w\_2, w\_3$: Bobot koefisien (ditetapkan dalam parameter Genesis).

Penggunaan logaritma natural ($\\ln$) memastikan bahwa lonjakan aktivitas ekstrim tidak merusak stabilitas protokol (*diminishing returns*).

---

## **3.3. The Time Dilation Formula (Effective Block Height)**

Sanctuary menggantikan penggunaan *Block Number* mentah ($t\_{raw}$) dalam kurva Sigmoid dengan **Effective Block Number** ($t\_{eff}$).

### **Proposisi 3.3 (Akumulasi Waktu Efektif)**

Setiap kali blok baru diproduksi, protokol tidak hanya menambahkan 1 ke penghitung waktu, melainkan menambahkan faktor akselerasi $\\alpha$.

$$t_{eff}^{(n)} = t_{eff}^{(n-1)} + \Delta t_{base} \times (1 + \alpha \cdot (Q_{net} - Q_{baseline}))$$
Dimana:

* $\Delta t_{base} = 1$ (Waktu standar).  
* $Q_{baseline}$: Target aktivitas jaringan standar.  
* $\alpha$: Konstanta sensitivitas (misalnya $0.05$).

Implikasi:

Jika $Q_{net} \> Q_{baseline}$ (Jaringan "Booming"), maka $t_{eff}$ bertambah lebih cepat dari 1\.

* *Contoh:* Dalam 1 hari real (24 jam), protokol mungkin menganggap telah berlalu 30 jam secara ekonomi.  
* *Hasil:* Kita bergerak lebih cepat menuju sisi kanan kurva Sigmoid (supply semakin langka).

---

## **3.4. Adaptive Reward Calculation**

Menggabungkan Bab 2 dan Bab 3, kita mendapatkan rumus final untuk **Reward Per Blok ($R\_b$)** yang dinamis.

Kita mensubstitusi $t$ dalam persamaan Sigmoid Bab 2 dengan $t_{eff}$:

$$R_b(t_{eff}) = k \cdot S(t_{eff}) \cdot \left(1 - \frac{S(t_{eff})}{S_{max}}\right)$$

### **Visualisasi Perilaku Protokol**

1. Skenario A (Bear Market / Adopsi Rendah):  
   $Q_{net}$ rendah. $t_{eff} \approx t_{raw}$. Kurva emisi berjalan lambat sesuai rencana standar (21 tahun). Validator tetap mendapatkan reward yang layak untuk menjaga keamanan jaringan.  
2. Skenario B (Bull Market / Adopsi Massal):  
   $Q_{net}$ tinggi. $t_{eff}$ melonjak. Protokol mempercepat pengurangan reward.  
   * Supply baru menjadi sangat langka tepat saat demand sedang tinggi.  
   * Ini menciptakan tekanan deflasi ganda: **High Demand \+ Accelerated Scarcity**.

---

## **3.5. Implementasi Teknis (Substrate Pallet)**

Di Substrate, logika ini ditanamkan pada hook on\_initialize atau on\_finalize di setiap pergantian Epoch (misal: setiap 1 hari) untuk menghemat komputasi (*Weight*).

**Pseudocode Rust Implementation:**

```Rust
// sanctuary-tokenomics/src/lib.rs

\#\[pallet::storage\]  
pub(super) type EffectiveBlockHeight\<T: Config\> \= StorageValue\<\_, u128, ValueQuery\>;

\#\[pallet::storage\]  
pub(super) type NetworkActivityQuotient\<T: Config\> \= StorageValue\<\_, FixedU128, ValueQuery\>;

impl\<T: Config\> Pallet\<T\> {  
      
    // Dijalankan setiap akhir Epoch (misal 1 Era \= 24 jam)  
    fn update\_time\_dilation(now\_block: T::BlockNumber) {  
        // 1\. Ambil metrik aktivitas periode ini  
        let tx\_volume \= T::Metrics::get\_transaction\_volume();  
        let unique\_stakers \= T::Staking::active\_counter();  
          
        // 2\. Hitung Q\_net (logarithmic dampening)  
        let q\_net \= Self::calculate\_q\_net(tx\_volume, unique\_stakers);  
          
        // 3\. Hitung Time Acceleration Factor  
        // alpha \= 0.05 (Sensitivity)  
        let alpha \= FixedU128::from\_rational(5, 100);   
        let baseline \= FixedU128::from(1); // Baseline activity index  
          
        let acceleration \= if q\_net \> baseline {  
             alpha.saturating\_mul(q\_net.saturating\_sub(baseline))  
        } else {  
             FixedU128::zero()  
        };

        // 4\. Update Effective Height  
        // t\_eff \= t\_prev \+ blocks\_passed \* (1 \+ acceleration)  
        let blocks\_passed \= now\_block \- Self::last\_update\_block();  
        let time\_dilated \= blocks\_passed.saturating\_mul(  
            FixedU128::one().saturating\_add(acceleration)  
        );  
          
        \<EffectiveBlockHeight\<T\>\>::mutate(|val| \*val \+= time\_dilated);  
    }  
}
```
---

## **3.6. Analisis Keamanan Game Theory**

Mekanisme ini memperkenalkan vektor serangan baru: **Inflation Griefing**.

* *Serangan:* Seorang "Whale" melakukan spam transaksi ($T_{vol}$ tinggi) dengan sengaja untuk meningkatkan $Q_{net}$ secara artifisial, dengan tujuan mempercepat pengurangan reward bagi validator lain (membuat mining/staking kurang profit bagi kompetitor).  
* *Mitigasi:* Biaya ($Cost$). Karena $w_3$ dalam rumus $Q_{net}$ memperhitungkan $B_{burned}$ (Fee Burn), biaya untuk melakukan spam transaksi agar $Q_{net}$ naik secara signifikan akan selalu lebih besar daripada kerugian marjinal validator akibat percepatan halving.  
* **Kesimpulan:** Sistem ini *Incentive Compatible*. Percepatan kelangkaan hanya menguntungkan pemegang token ($\$SANC$ holder) karena apresiasi harga, bukan penyerang yang membuang uang untuk fee.

---

# **CHAPTER 4: CRYPTOGRAPHIC ARCHITECTURE (IDENTITY & SECURITY)**

## **4.1. The Quantum Threat Model**

Sistem kriptografi kunci publik yang digunakan oleh Bitcoin dan Ethereum (ECDSA pada kurva secp256k1) menyandarkan keamanannya pada kesulitan memecahkan *Discrete Logarithm Problem*. Algoritma Shor, dijalankan pada komputer kuantum dengan qubit yang cukup logis, dapat memecahkan masalah ini dalam waktu polinomial, secara efektif memungkinkan penurunan *Private Key* dari *Public Key* yang terekspos.

Sanctuary Protocol mengadopsi pendekatan **"Proactive Defense"**: Alih-alih menunggu ancaman muncul, protokol dibangun dengan asumsi bahwa musuh sudah memiliki kapabilitas kuantum.

---

## **4.2. Hybrid Signature Scheme (Dual-Stack Identity)**

Untuk menyeimbangkan adopsi pengguna (yang terbiasa dengan wallet EVM) dan keamanan jangka panjang, Sanctuary mengimplementasikan skema tanda tangan hibrida di level primitif Substrate.

### **Definisi 4.2.1 (Sanctuary MultiSignature)**

Kami memperluas tipe data tanda tangan standar Substrate menjadi enum yang mendukung dua skema kriptografi secara simultan:

1. **Legacy Scheme ($Sig\_{eth}$):** ECDSA secp256k1 \+ Keccak256 hashing. (Kompatibel penuh dengan Ethereum/Metamask).  
2. **Sanctuary Scheme ($Sig\_{pqc}$):** **ML-DSA (Module-Lattice-Based Digital Signature Algorithm)**, sebelumnya dikenal sebagai **CRYSTALS-Dilithium**. Standar FIPS 204 dari NIST.

Struktur Rust dalam Runtime:

```Rust
\#\[derive(Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug)\]  
pub enum SanctuarySignature {  
    /// Tanda tangan standar Ethereum (65 bytes).   
    /// Rentan terhadap serangan kuantum, tapi cepat dan hemat gas.  
    Ecdsa(sp\_core::ecdsa::Signature),  
      
    /// Tanda tangan Post-Quantum (2420 bytes untuk Dilithium3).  
    /// Aman secara matematis dari serangan kuantum.  
    Dilithium(sp\_core::dilithium::Signature),   
}
```
### **Definisi 4.2.2 (Alamat Wallet)**

Alamat di Sanctuary (AccountId) bersifat agnostik terhadap jenis kunci.

* Alamat adalah *hash* 32-byte (Blake2) dari Public Key.  
* Satu akun bisa dikendalikan oleh kunci ECDSA, lalu di-*upgrade* untuk dikendalikan oleh kunci Dilithium, tanpa mengubah alamat wallet. Ini memungkinkan persistensi reputasi dan saldo.

---

## **4.3. The Quantum Upgrade Protocol (QUP)**

Ini adalah fitur paling kritis bagi *user safety*. QUP memungkinkan pengguna memulai dengan Metamask (mudah), lalu "mengunci" akun mereka dengan keamanan PQC (aman) saat aset mereka bernilai tinggi.

### **Mekanisme Transisi (The Bonding Process)**

Proses ini dilakukan melalui transaksi on-chain khusus yang disebut bond\_quantum\_key.

1. **Generasi:** Pengguna menghasilkan pasangan kunci Dilithium $(Pk_{pqc}, Sk_{pqc})$ secara lokal (client-side).  
2. **Asosiasi:** Pengguna mengirim transaksi yang ditandatangani oleh kunci ECDSA lama ($Sk_{eth}$), yang berisi payload $Pk_{pqc}$.  
   $$Tx = Sign(Sk_{eth}, \text{"Upgrade to: "} + Pk_{pqc})$$
3. **Finalisasi (The Lock):** Setelah transaksi ini diverifikasi dan dimasukkan ke dalam blok, Runtime Sanctuary memutar "saklar keamanan" (SecurityFlag) untuk akun tersebut.

### **Proposisi 4.3 (Immutability of Upgrade)**

Setelah SecurityFlag aktif:

1. Protokol **MENOLAK** semua transaksi dari akun tersebut yang ditandatangani dengan ECDSA.  
2. Hanya tanda tangan Dilithium yang valid.

Implikasi Keamanan:

Jika di masa depan komputer kuantum berhasil meretas kunci ECDSA lama pengguna, peretas tetap tidak bisa mencuri dana, karena protokol telah menganggap kunci ECDSA tersebut usang (deprecated) untuk akun itu. Kunci ECDSA hanya menjadi "kunci pintu depan yang sudah diganti lubang kuncinya".

---

## **4.4. The Sentinel Module (Intrusion Detection System)**

Sanctuary menyertakan modul Runtime khusus bernama pallet-sentinel yang berfungsi sebagai "Penjaga Gerbang" heuristik.

### **Fungsi 4.4.1 (Signature Policy Enforcement)**

Sentinel mencegat setiap transaksi sebelum masuk ke *Transaction Pool*.

```Rust

fn validate\_transaction(who: \&AccountId, signature: \&SanctuarySignature) \-\> Result {  
    let account\_info \= System::account(who);  
      
    // Cek apakah user sudah upgrade ke PQC  
    if account\_info.security\_level \== SecurityLevel::Quantum {  
        match signature {  
            SanctuarySignature::Dilithium(\_) \=\> return Ok(()),  
            SanctuarySignature::Ecdsa(\_) \=\> {  
                // TOLAK KERAS.   
                // Ini bisa jadi upaya replay attack atau serangan kuantum pada kunci lama.  
                return Err(InvalidTransaction::StaleKey);  
            }  
        }  
    }  
      
    // Jika masih Legacy level, izinkan keduanya  
    Ok(())  
}
```
### **Fungsi 4.4.2 (Entropy Monitoring)**

Sentinel memantau entropi tanda tangan. Meskipun serangan kuantum sulit dideteksi secara langsung, upaya pengumpulan data publik (*harvesting public keys*) yang masif dapat dideteksi. Sentinel memiliki wewenang untuk menaikkan *Base Fee* secara drastis jika mendeteksi anomali pada verifikasi tanda tangan, membuat serangan spam/brute-force menjadi tidak ekonomis.

---

## **4.5. Key Encapsulation Mechanism (KEM) untuk Privasi**

Selain tanda tangan (untuk transaksi), Sanctuary juga menyediakan primitif untuk pesan terenkripsi on-chain yang aman dari kuantum.

Menggunakan ML-KEM (Kyber) \- Standar FIPS 203\.

Ini memungkinkan fitur masa depan seperti:

* **Quantum-Safe DAO Voting:** Isi suara dienkripsi, hanya bisa dibuka setelah voting selesai.  
* **Encrypted State:** Menyimpan data rahasia di smart contract yang tidak bisa didekripsi oleh komputer kuantum di masa depan (mencegah strategi *"Store now, decrypt later"*).

---

# **CHAPTER 5: CONSENSUS & FINALITY (THE ENGINE)**

## **5.1. The Hybrid Consensus Model**

Untuk mencapai trilema blockchain (Keamanan, Skalabilitas, Desentralisasi), Sanctuary memisahkan proses produksi blok dari proses pengamanan blok. Kami menggunakan arsitektur konsensus hibrida yang membagi tanggung jawab kepada dua protokol terpisah:

1. **BABE (Blind Assignment for Blockchain Extension):** Protokol probabilistik untuk produksi blok yang cepat (The Writer).  
2. **GRANDPA (GHOST-based Recursive Ancestor Deriving Prefix Agreement):** Gadget finalitas deterministik untuk keamanan absolut (The Judge).

Pemisahan ini memungkinkan Sanctuary untuk terus memproduksi blok meskipun jaringan tidak stabil, namun tetap menjamin bahwa sejarah transaksi tidak akan pernah bisa diubah (*reorg*) setelah difinalisasi.

---

## **5.2. Block Production: BABE (The Rhythm)**

BABE membagi waktu menjadi segmen diskrit yang disebut **Epoch** dan **Slot**. Setiap slot berdurasi tepat 6 detik (target block time).

### **Mekanisme 5.2.1 (Verifiable Random Function / VRF)**

Siapa yang berhak memproduksi blok selanjutnya? Tidak ada yang tahu sampai saatnya tiba. Validator menjalankan fungsi acak kriptografis (VRF) secara lokal.

$$Proof = VRF(Sk\_{validator}, Randomness\_{epoch}, Slot\_{number})$$

Jika $Proof < Threshold$, validator tersebut berhak memproduksi blok pada slot tersebut.

* **Keamanan:** Karena bersifat "Blind", penyerang tidak bisa melakukan DDoS pada produsen blok berikutnya karena identitas mereka baru terungkap saat blok dipublikasikan.  
* **Persyaratan PQC:** Di Sanctuary, input untuk VRF tetap menggunakan kurva elliptik (Schnorrkel/Ristretto) untuk kecepatan, namun blok yang dihasilkan **wajib** ditandatangani ganda menggunakan kunci Post-Quantum (Dilithium) yang didefinisikan di Bab 4 untuk dianggap valid oleh jaringan.

---

## **5.3. Finality: GRANDPA (The Anchor)**

Berbeda dengan Ethereum (Casper) atau Cosmos (Tendermint) yang memfinalisasi blok satu per satu, GRANDPA memfinalisasi **rantai (chain)**.

### **Proposisi 5.3 (Chain-Based Voting)**

Validator memberikan suara bukan pada "Blok X", tapi pada "Blok X adalah blok tertinggi yang saya anggap valid". Secara transitif, logika ini memvalidasi semua blok leluhur (ancestors) dari Blok X.

* **Kecepatan Teoretis:** Jika terjadi partisi jaringan dan kemudian pulih, GRANDPA dapat memfinalisasi jutaan blok sekaligus dalam satu putaran voting.  
* **Jaminan Keamanan:** Setelah 2/3 \+ 1 validator memberikan suara *pre-commit* pada sebuah rantai, probabilitas *revert* adalah nol. Ini memberikan kepastian hukum bagi transaksi finansial besar.

---

## **5.4. Validator Selection: NPoS & The Phragmén Method**

Bagian ini adalah jawaban Sanctuary terhadap sentralisasi. Kebanyakan sistem PoS (seperti BSC atau EOS) menggunakan DPoS dimana "Winner Takes All", menyebabkan hanya paus besar yang menjadi validator.

Sanctuary menggunakan **Nominated Proof of Stake (NPoS)** dengan algoritma optimasi **Sequential Phragmén**.

### **Masalah Optimasi 5.4.1 (Fair Representation)**

Tujuannya adalah memilih himpunan validator $V$ dari kandidat $C$ sedemikian rupa sehingga *stake* yang mendukung mereka terdistribusi serata mungkin.

Kami meminimalkan varians dari *backing stake* antar validator terpilih. Tujuannya adalah memaksimalkan skor keamanan jaringan $S$:

$$S = \min_{v \in V} (\text{TotalStake}(v))$$
Dimana fungsi ini mencoba memastikan validator terkecil pun memiliki dukungan *stake* yang cukup besar, sehingga sulit diserang.

### **Diagram Alur Distribusi Stake**

1. **Nominator** (pemilik token) memilih hingga 16 kandidat yang mereka percaya.  
2. **Algoritma Phragmén** berjalan di akhir setiap Era (24 jam).  
3. Algoritma secara otomatis mendistribusikan token nominator ke kandidat terpilih untuk menyeimbangkan beban.

**Hasil:** Matematika Phragmén mencegah pembentukan kartel secara alami. Jika sekelompok Paus mencoba memusatkan suara pada satu validator, algoritma akan memindahkan suara nominator kecil ke validator lain untuk menyeimbangkan jaringan.

---

## **5.5. Economic Security & Slashing**

Keamanan Sanctuary bukan hanya kriptografi, tapi juga **Game Theory**. Validator harus mempertaruhkan aset ekonomi ($SANC) yang akan dimusnahkan jika mereka bertindak jahat.

### **Aturan 5.5.1 (Slashing Conditions)**

Terdapat dua jenis pelanggaran utama:

1. **Unresponsiveness (Pasif):** Validator offline dan tidak memproduksi blok.  
   * *Sanksi:* Pemotongan kecil (*Chilling*). Dikeluarkan dari set aktif periode berikutnya.  
2. **Equivocation (Jahat):** Validator menandatangani dua blok berbeda pada slot yang sama (*Double Signing*) atau memberikan suara kontradiktif di GRANDPA.  
   * *Sanksi:* **100% Slash**. Sebagian besar stake dibakar (dikirim ke Treasury atau Blackhole Address), identitas PQC diban permanen.

### **Integrasi dengan Adaptive Scarcity (Bab 3\)**

Reward yang diterima validator ($R_{val}$) diambil dari emisi per blok yang dihitung di Bab 3\.

$$R_{val} = R_b(t_{eff}) \times (1 - \text{TreasuryCut})$$
Validator dibayar menggunakan mata uang yang semakin langka seiring waktu. Ini menciptakan insentif jangka panjang yang sangat kuat untuk menjaga integritas jaringan, karena merusak jaringan berarti menghancurkan nilai aset yang mereka kumpulkan.

---

# **CHAPTER 6: THE VIRTUAL MACHINE (EXECUTION LAYER)**

## **6.1. The Sanctuary EVM (SEVM) Architecture**

Sanctuary Protocol mengadopsi pendekatan pragmatis terhadap eksekusi *Smart Contract*. Meskipun *native runtime* Sanctuary ditulis dalam Rust dan dikompilasi ke WebAssembly (WASM) untuk performa maksimal, kami menyadari bahwa Ethereum Virtual Machine (EVM) adalah standar industri saat ini.

Oleh karena itu, Sanctuary mengimplementasikan **SEVM (Sanctuary EVM)** menggunakan modul pallet-evm (Frontier) yang berjalan di atas Substrate.

### **Definisi 6.1.1 (Emulasi Full-State)**

SEVM bukanlah *sidechain* atau *layer-2*. Ia adalah lingkungan eksekusi penuh yang berjalan di dalam logika konsensus Layer-1.

* **Opcode Compatibility:** Mendukung seluruh set instruksi EVM (Shanghai/Cancun update).  
* **Environment:** Kode Solidity (.sol) yang dikompilasi dengan solc standar dapat di-deploy tanpa modifikasi sedikitpun.

---

## **6.2. Unified Address & Account System**

Tantangan terbesar dalam blockchain hibrida adalah fragmentasi akun (memiliki dua alamat berbeda untuk satu user). Sanctuary memecahkan ini dengan **Unified Account Model**.

### **Mekanisme 6.2.1 (Address Mapping)**

Sanctuary memetakan alamat Ethereum (H160 \- 20 bytes) ke dalam ruang alamat Substrate (H256 \- 32 bytes) secara transparan.

* **Alamat H160:** 0x71C765... (Digunakan di Metamask/EVM).  
* **Alamat Substrate:** Dikonversi melalui hashing prefix tertentu sehingga satu *Private Key* mengontrol saldo di kedua sisi (Native Staking & EVM Contract interaction).

User tidak perlu melakukan "bridging" atau memindahkan aset antar layer. $SANC di dompet Metamask adalah $SANC yang sama yang digunakan untuk Staking di konsensus NPoS.

---

## **6.3. Quantum-Native Precompiles (The Bridge to Math)**

Inilah "Senjata Rahasia" Sanctuary. Standar EVM tidak mengetahui tentang *Adaptive Scarcity* atau *Post-Quantum Cryptography*. Untuk menjembatani ini, kami membuat **Precompiled Contracts** khusus.

Developer Solidity dapat memanggil fitur canggih Sanctuary seolah-olah memanggil fungsi smart contract biasa di alamat tertentu.

### **Precompile A: The Oracle of Constants (0x...00AA)**

Kontrak ini mengekspos data ekonomi makro protokol ke dApps (DeFi).

Contoh Use Case:

Sebuah protokol Lending (pinjam-meminjam) ingin menyesuaikan suku bunga berdasarkan "Kesehatan Jaringan" ($Q\_{net}$) yang didefinisikan di Bab 3\.

```solidity
interface ISanctuaryOracle {  
    // Mengambil nilai Q\_net saat ini (Network Activity)  
    function getNetworkQuotient() external view returns (uint256);  
      
    // Mengambil total supply real-time berdasarkan kurva Sigmoid  
    function getCurrentSupply() external view returns (uint256);  
}

contract AdaptiveDeFi {  
    ISanctuaryOracle constant ORACLE \= ISanctuaryOracle(0x00000000000000000000000000000000000000AA);

    function calculateInterestRate() public view returns (uint256) {  
        uint256 activity \= ORACLE.getNetworkQuotient();  
        if (activity \> 1000\) {  
            // Jika jaringan sibuk (High Demand), naikkan bunga  
            return 500; // 5%  
        }  
        return 200; // 2%  
    }  
}
```
### **Precompile B: The Quantum Verifier (0x...00BB)**

Kontrak ini memungkinkan dApp memverifikasi tanda tangan Dilithium (PQC) secara on-chain.

Contoh Use Case:

Sebuah DAO ingin membuat Treasury yang hanya bisa dibuka dengan persetujuan kunci kuantum (Ultra-Secure Multisig).

```solidity
interface IQuantumVerifier {  
    function verifyDilithium(  
        bytes memory message,   
        bytes memory signature,   
        bytes memory publicKey  
    ) external view returns (bool);  
}
```
---

## **6.4. Gas Metering & Weight**

Sanctuary menerjemahkan konsep **Weight** (satuan komputasi Substrate) menjadi **Gas** (satuan komputasi Ethereum) secara dinamis.

* **Biaya Deterministik:** Karena blok diproduksi setiap 6 detik (tetap), estimasi gas jauh lebih stabil dibandingkan Ethereum mainnet.  
* **EIP-1559 Support:** Sanctuary mengadopsi mekanisme pembakaran fee (*Base Fee Burn*) yang kompatibel dengan EIP-1559. Fee yang dibakar ini berkontribusi langsung pada variabel $B_{burned}$ di Bab 3, yang selanjutnya mempengaruhi kecepatan kelangkaan supply.

Siklus Umpan Balik:

Transaksi EVM tinggi $\rightarrow$ Fee Burn naik $\rightarrow$ $Q_{net}$ naik $\rightarrow$ Halving dipercepat $\rightarrow$ $\$SANC$ semakin langka.

---

## **6.5. Developer Experience (DX)**

Visi Sanctuary adalah "Zero Friction Migration".

1. **RPC Endpoint:** Kompatibel penuh dengan JSON-RPC Ethereum.  
2. **Tooling:**  
   * **Metamask:** Cukup tambahkan "Custom Network".  
   * **Remix IDE:** Deploy langsung ke Sanctuary.  
   * **Hardhat/Foundry:** Gunakan script deployment yang sudah ada.  
   * **The Graph:** Mendukung indeksasi data on-chain.

Developer tidak perlu menginstal Rust atau memahami Substrate untuk membangun aplikasi di atas Sanctuary. Mereka hanya perlu tahu bahwa aplikasi mereka berjalan di atas infrastruktur yang paling aman secara matematis di dunia.

---

# **CONCLUSION: THE SANCTUARY MANIFESTO**

Dokumen ini, *The Sanctuary Protocol Yellow Paper*, telah menguraikan desain teknis untuk sebuah sistem moneter digital yang:

1. **Alamiah:** Menggunakan konstanta universal ($\pi, e, \phi$) untuk menentukan supply, bukan kebijakan manusia.  
2. **Adaptif:** Bernapas bersama aktivitas jaringan, menyesuaikan kelangkaan secara organik.  
3. **Abadi:** Dibentengi oleh kriptografi Post-Quantum (Dilithium) yang tahan terhadap ancaman komputasi masa depan.  
4. **Adil:** Mendistribusikan kekuasaan melalui konsensus NPoS dan algoritma Phragmén.  
5. **Terbuka:** Menyediakan utilitas tanpa batas melalui kompatibilitas EVM penuh.

Sanctuary bukan sekadar "The Next Bitcoin" atau "Ethereum Killer". Sanctuary adalah evolusi logis berikutnya dari uang digital: **Mathematics-as-Money.**

---

[image1]: <data:image/png;base64,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>