# PostgreSQL 18 UUIDv7 機能調査

## 概要

PostgreSQL 18では、UUIDv7のネイティブサポートが追加されます。本ドキュメントでは、PostgreSQL 18で実装されるUUIDv7機能と、pgx_uuidv7拡張との比較を行います。

## PostgreSQL 18のUUIDv7実装

### 追加される関数

1. **`uuidv7()`**
   - 現在時刻でUUIDv7を生成
   - パラメータなし（RFC 9652準拠のため、カスタムタイムスタンプは受け付けない）
   - 戻り値: `uuid`型

2. **`uuidv7(interval)`**
   - 現在時刻に指定されたインターバルを加算した時刻でUUIDv7を生成
   - パラメータ: `interval`型
   - 戻り値: `uuid`型
   - PostgreSQL 18で新規追加

3. **`uuidv4()`**
   - `gen_random_uuid()`のエイリアス
   - 明示的にバージョン4のUUIDを生成
   - 戻り値: `uuid`型

4. **`uuid_extract_timestamp(uuid)`**
   - UUID v1とv7からタイムスタンプを抽出
   - PostgreSQL 18でv7サポートが追加
   - 戻り値: `timestamp with time zone`
   - 他のバージョン（v4など）ではNULLを返す

5. **`uuid_extract_version(uuid)`**
   - UUIDのバージョン番号を抽出
   - 全バージョン対応（v1-v7）
   - 戻り値: `smallint`（バージョン番号）

### タイムスタンプ精度

- **基本精度**: ミリ秒（48ビット、RFC 9652準拠）
- **追加精度**: 12ビットの`rand_a`フィールドにサブミリ秒のタイムスタンプを格納
- **単調性保証**: 同一バックエンド内で生成されるUUIDは単調増加を保証
  - システムクロックが逆行した場合でも対応
  - 高頻度生成（1000件/秒以上）でも順序性を維持

### 実装の特徴

1. **RFC 9652準拠**: 最新のUUID仕様に完全準拠
2. **ソート可能性**: タイムスタンプベースのため時系列でソート可能
3. **インデックス効率**: 時系列順序により良好なインデックスローカリティ
4. **ユニーク性**: 74ビットのランダム部分により衝突を防止

## pgx_uuidv7との機能比較

| 機能 | PostgreSQL 18 | pgx_uuidv7 |
|------|--------------|------------|
| **UUID生成** |
| 現在時刻でのUUID生成 | `uuidv7()` | `uuid_generate_v7_now()` |
| インターバル付きUUID生成 | `uuidv7(interval)` | `uuid_generate_v7_at_interval(interval)` |
| 指定時刻でのUUID生成 | ❌ | `uuid_generate_v7(timestamptz)` |
| **時刻変換** |
| UUIDから時刻抽出 | `uuid_extract_timestamp()` | `uuid_to_timestamptz()` |
| 時刻の最小UUID生成 | ❌ | `timestamptz_to_uuid_v7_min()` |
| 時刻の最大UUID生成 | ❌ | `timestamptz_to_uuid_v7_max()` |
| **その他の機能** |
| バージョン取得 | `uuid_extract_version()` | `uuid_get_version()` |
| CAST演算子 | ❌ | `uuid::timestamptz` |
| **タイムスタンプ精度** |
| 基本精度 | ミリ秒 | ミリ秒 |
| サブミリ秒精度 | ✅ (rand_aフィールド使用) | ❌ |
| 単調性保証 | ✅ (同一バックエンド内) | ❌ |

### 関数の詳細な違い

#### uuid_extract_version vs uuid_get_version

| 項目 | PostgreSQL 18 | pgx_uuidv7 |
|------|--------------|------------|
| 関数名 | `uuid_extract_version()` | `uuid_get_version()` |
| 戻り値の型 | `smallint` (16ビット) | `i8` (8ビット) |
| 対応バージョン | 全バージョン (v1-v7) | 全バージョン (v1-v7) |
| NULL処理 | RFC 4122/9562以外の場合NULL | N/A |

**備考**: UUIDのバージョン番号は1〜8の範囲なので、実用上は`i8`で十分ですが、PostgreSQL 18は将来の拡張性を考慮して`smallint`を採用。

#### uuid_extract_timestamp vs uuid_to_timestamptz

| 項目 | PostgreSQL 18 | pgx_uuidv7 |
|------|--------------|------------|
| 関数名 | `uuid_extract_timestamp()` | `uuid_to_timestamptz()` |
| 戻り値の型 | `timestamp with time zone` | `timestamp with time zone` |
| 対応UUIDバージョン | v1, v7 | 主にv7 (uuid crateに依存) |
| NULL処理 | v1/v7以外でNULL返却 | タイムスタンプ抽出不可時NULL |
| タイムゾーン | UTC | UTC |
| 実装 | ネイティブC | Rust (uuid crate使用) |

**備考**: PostgreSQL 17以前は`uuid_extract_timestamp`はv1のみ対応。PostgreSQL 18でv7サポートが追加された。

## 移行戦略

### PostgreSQL 17以前
- pgx_uuidv7拡張を使用
- 全機能が利用可能

### PostgreSQL 18
- **基本的なUUID生成**: 標準の`uuidv7()`関数を使用
- **拡張機能が必要な場合**:
  - 任意の時刻でのUUID生成
  - 時刻範囲でのUUID検索（最小/最大UUID）
  - 暗黙的な型変換（CAST）

### 推奨事項

1. **新規プロジェクト（PostgreSQL 18）**:
   - 基本的には標準の`uuidv7()`を使用
   - 高度な機能が必要な場合のみpgx_uuidv7を検討

2. **既存プロジェクトの移行**:
   - 関数名の違いに注意（`uuid_generate_v7_now()` → `uuidv7()`）
   - 時刻変換機能を使用している場合は、pgx_uuidv7の継続使用を検討

3. **互換性の確保**:
   - pgx_uuidv7で標準関数のラッパーを提供することを検討
   - 将来的には、標準にない機能のみに特化

## まとめ

PostgreSQL 18のUUIDv7実装は、基本的な生成機能に焦点を当てた堅牢な実装です。特にサブミリ秒精度と単調性保証により、高頻度でのUUID生成においても優れた性能を発揮します。

一方、pgx_uuidv7は時刻変換や範囲検索などの追加機能を提供しており、これらの機能が必要なユースケースでは引き続き価値があります。両者は補完的な関係にあり、要件に応じて使い分けることが推奨されます。