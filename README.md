---
title: Actor model by Rust with Actix
tags: Rust actix async actor
author: atsuk
slide: false
---
当記事ではActix frameworkについての概要と使い方について説明します。

# 前提
actix: 0.7.3

# Actix
ActixとはRustにおけるアクターモデルのフレームワークです。
アクターモデルの詳細については触れませんが、それぞれが専用のメールボックスを持ち、並行して非同期で動作するアクターと呼ばれる構造体群を起動させて、アクター同士でメッセージを送受信して処理を行う方式になります。

アクターとメッセージがオブジェクト指向におけるクラスとメソッドに似ていますが、アクターは並行性を兼ねている点が異なります。
メッセージは個々のアクターが持つメールボックスにキューイングされて、アクターが処理可能なタイミングで一つずつ取得して処理を行います。
そのため、アクターの処理自体はマルチスレッドにおけるロックなどの排他処理は考えずにシングルスレッドで構築することが出来ます。
また、アクター自体はどこかのスレッド上で動作しているのですが、メッセージパッシングの観点からは動作している場所を気にせずにメッセージを送信して処理するため、スレッドやロックについてあまり考えずに並行処理を組み立てられる点が大きな利点だと考えています。

アクターモデル実装はErlang/Elixirや、Scala/JavaにおけるAkkaが有名ですが、私がそれらを実際に触ったことが無いので比較等はできません。
認識している限りではまだリモートサーバー実行が出来ない(*1)等、Akka等と比較すると機能は少ない点があるようです。

よって、現状Actixが有効に作用するものとしては、シングルサーバー、シングルプロセスで動作するマルチスレッドを活かしたアプリケーションとなります。

尚、actix-webというWeb frameworkはこのActixシステムの上で構築されており、actix-webでActixによるアクターモデルを使用することも可能です。

*1) [actix-remote](https://github.com/actix/actix-remote)というリモートサーバーで使用できるようにするためのリポジトリは在りますが、開発は進んでいないようです。

# System
`System`はActixにおけるランタイムとなります。
アクターは`System`が作成されるまでは起動することは出来ません。
`System::new`を呼び出すと`SystemRunner`が返却されますが、アクターが動作する場所となるイベントループを開始して処理をブロックするために`SystemRunner`の`run`メソッドを実行する必要があります。

イベントループを停止するには起動しているSystemインスタンスの`stop`メソッドを呼び出す必要があります。
現在の`System`インスタンス自体はActixイベントループ上であれば`System::current`で取得できるため、`System::current().stop()`という一連の処理で停止出来ます。

アクターを開始してメッセージを出力し、すぐに処理を終了する簡単なコード例を以下に記載します。

```rust:main.rs
extern crate actix;

use actix::prelude::*;

// Testアクター構造体
struct Test;

// Actorトレイト実装
impl Actor for Test {
    type Context = Context<Self>;

    // Testアクター開始時に呼ばれる処理
    fn started(&mut self, _ctx: &mut Self::Context) {
        println!("started");

        // System停止処理
        System::current().stop();
    }
}

fn main() {
    // System作成
    let system = System::new("test");

    // Testアクター開始
    Test.start();

    // イベントループを開始し、System停止処理が呼ばれるまでブロックする
    system.run();
}
```

# Actor
Actixでは`Actor`トレイトを実装した構造体がアクターになります。
アクターは全て独立した実行コンテキスト上で動作します。
`Actor`トレイト実装時には関連型としてコンテキストを定義する必要があります。
現在設定できるのは`Context<A>`か`SyncContext<A>`の二種類となっています。
後述する`SyncArbiter`を使う場合は`SyncContext<A>`、それ以外は`Context<A>`を使う形になります。

## Start
アクターを開始するためのメソッドとして以下3種類があります。

* `start`
    * プログラムで作成した対象アクターを開始します。
* `start_default`
    * Defaultトレイトが実装されている対象アクターをデフォルトで作成して開始します。
* `create`
    * Contextを受け取って対象アクターを返却する関数を受け取り、その実行結果のアクターで開始します。

上記全てアドレス`Addr<A>`を戻り値とし、このアドレスを使用してメッセージ送信を行うことが出来ます。
メッセージについては後述するMessage欄で説明します。
これらのメソッドで開始した場合、メソッド実行時の`Arbiter`上で動作します。
`Arbiter`については後述しますが、メソッド実行時と同一スレッド/同一イベントループで動作することとなります。

## Lifecycle
アクターにはStarted, Running, Stopping, Stoppedの4種類の状態（ライフサイクル）があります。
アクター起動直後はStartedとなり、`Actor`トレイトの`started`メソッドが呼ばれます。
`started`メソッド呼び出しが完了するとRunningに移行します。
Running状態ではメッセージの受信が可能になります。

Running状態で以下の何れかが発生した際、Stopping状態に移行し、`stopping`メソッドが呼ばれます。
`stopping`メソッドではそのままStopped状態に移行するか、Running状態に移行するのかを戻り値で選択することが出来ます。デフォルトではStopped状態に移行となります。

* 対象アクター自身によって`Context::stop`が呼び出される
* 対象アクターに対する全てのアドレスがドロップされる
* Contextにイベントが登録されていない

Stopped状態に移行すると`stopped`メソッドが呼ばれます。
`stopped`メソッド処理が終了すると、対象アクターはドロップされます。
尚、`stopped`メソッド内でContextに非同期処理を登録しても実行されずにドロップされます。

## サンプルコード
ここまで説明した内容によるサンプルコードを記載します。

```rust:main.rs
extern crate actix;

use actix::prelude::*;

#[derive(Default)]
struct Test;

impl Actor for Test {
    type Context = Context<Self>;

    fn started(&mut self, _ctx: &mut Self::Context) {
        println!("started");
    }

    fn stopping(&mut self, _ctx: &mut Self::Context) -> Running {
        println!("stopping");
        Running::Stop
    }

    fn stopped(&mut self, _ctx: &mut Self::Context) {
        println!("stopped");
        System::current().stop();
    }
}

fn main() {
    let system = System::new("test");

    // 戻り値のAddr<Test>を保持しないためすぐにdropされ、Testアクターはstopping状態に移行します
    Test.start();
    // Test::start_default();
    // Test::create(|_| Test);

    system.run();
}
```

## Context
Contextはアクターの実行コンテキストを表します。
実行コンテキストはアクター単位で独立しています。
アクターを停止したり、非同期処理を行う際に使用します。

いくつかのメソッドを紹介します。

* `set_mailbox_capacity`
    * メールボックスのサイズを変更します。デフォルトは16です。
* `stop`
    * アクターをStopping状態に移行します。
* `terminate`
    * アクターをStopped状態に移行します。
* `address`
    * 自アクターの`Addr<A>`を取得します。
* `spawn`
    * 渡された`ActorFuture`を実行します。実行をキャンセルするための`SpawnHandle`を返却します。
* `cancel_future`
    * `SpawnHandle`を受け取り、対象非同期処理をキャンセルします。
* `wait`
    * 渡された`ActorFuture`を実行します。この非同期処理が完了するまでこの実行コンテキストに発生するイベントは待たされます。
    * 例として`started`内で非同期初期化を行うさいにこのメソッドを使用すると、初期化が完了するまでメッセージは処理されずに待たされる状況にすることが出来ます。
* `notify`
    * 自分自身にメッセージを送信します。
* `add_stream`
    * 非同期で複数のデータが返却される`Stream`を登録します。これを使用するアクターは対象`Stream`の返却Item及びErrorに該当する`StreamHandler`トレイトを実装する必要があり、`StreamHandler::handle`メソッドで処理されることになります。
* `add_message_stream`
    * `add_stream`に似ていますがエラーは無視されます。また、返却するItemは`Message`トレイトを実装している必要があります。`StreamHandler`は実装不要で、返却される`Message`を処理する`Handler`で処理されます。

## ActorFuture
Actixでは利便性の為に`Future`にアクター自身と`Context`を紐付けた`ActorFuture`が用意されています。
`self`や`Context`には`Send`が実装されていないため、通常の`Future`を使用した非同期処理を`Context`で実行した際の結果を持って`self`や`Context`を操作することが出来ません。
そこで、対象の`Future`を`ActorFuture`に変換することで`map`や`then`に渡す関数で非同期結果と共に`self`及び`Context`を受け取り、操作することが出来るようになっています。

`Future`には`WrapFuture`トレイトが実装されているため、`into_actor`メソッドを使用して`ActorFuture`に変換することが可能です。

## サンプルコード
ここまで説明した内容によるサンプルコードを記載します。
TCP接続して文字列を送信すると、接続している全員にブロードキャストされるチャットサーバーです。

```rust
extern crate actix;
extern crate tokio;

use std::{
    collections::HashMap,
    io::BufReader,
    net::SocketAddr
};

use actix::prelude::*;
use tokio::{
    prelude::*,
    io,
    net::{TcpListener, TcpStream}
};

#[derive(Default)]
struct Chat {
    writers: HashMap<SocketAddr, io::WriteHalf<TcpStream>>
}

impl Actor for Chat {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        println!("started");
        // TCPサーバー起動
        let socket = "127.0.0.1:3000".parse().unwrap();
        let listener = TcpListener::bind(&socket).unwrap();
        ctx.add_stream(listener.incoming());
    }
}

impl StreamHandler<TcpStream, io::Error> for Chat {
    fn handle(&mut self, stream: TcpStream, ctx: &mut Self::Context) {
        println!("Incoming user.");

        // キーとして接続ユーザーのソケットアドレスを取得
        let remote_addr = stream.peer_addr().unwrap();
        // streamをreaderとwriterに分割
        let (reader, writer) = stream.split();
        // writerを保持
        self.writers.insert(remote_addr.clone(), writer);

        // linesを使用してreaderから入力されるデータを行単位StringのStreamに変換
        let task = io::lines(BufReader::new(reader))
            .map_err(|e| println!("Read error. [{}]", e))
            .into_actor(self) // ActorStreamに変換
            .map(|mut line, actor, _| {
                // 行単位で入力処理
                println!("Lines. [{}]", line);
                // 改行を付与してbyte列に変換
                line.push('\n');
                let send_data = line.into_bytes();

                // 入室者全員にブロードキャスト (同期処理)
                for writer in actor.writers.values_mut() {
                    if let Err(e) = writer.write(&send_data) {
                        println!("Broadcast error. [{}]", e);
                    }
                }
            })
            .finish() // ActorStreamをActorFutureに変換
            .then(move |_, actor, _| {
                // 退室処理
                println!("Leave user.");
                actor.writers.remove(&remote_addr);
                actix::fut::ok(())
            });

        ctx.spawn(task);
    }
}

fn main() {
    let system = System::new("test");
    let _addr = Chat::start_default();
    system.run();
}
```

# Address
`Addr<A>`はアクターに対する参照となっており、アクターに送信するメッセージはこのアドレス、又は後述する`Recipient<M>`を使用して送信することが出来ます。
アドレスはクローン可能であり、`Send`、`Sync`が実装されているためスレッドを跨いで引き渡すことが可能です。
アドレスは前述したアクター開始メソッドの戻り値、及び`Context`の`address`メソッド、又は後述する`Registry`等から取得することが出来ます。

`Addr<A>`では3つのメッセージ送信メソッドが用意されています。

* `do_send`
    * メールボックス容量を無視してメッセージを送信します。メールボックスがクローズしていた場合は何も起きません。メッセージの戻り値は取得できません。
* `try_send`
    * メールボックス容量がフルであったり、クローズしていた場合は`Err`が返ります。メッセージの戻り値は取得できません。
* `send`
    * メッセージの戻り値を持つ`Future`が返却され、非同期処理の後続処理を記述出来ます。
    * 注意点として、返却される`Future`の`Item`がメッセージの戻り値、`Error`はメッセージ送信のエラーを表す`MailboxError`となります。例として送信メッセージの戻り値が`Result`の場合は、`Item`として`Result`が返却される形となります。

## Recipient
`Recipient<M>`は`Addr<A>`の特殊系であり、`Addr<A>`の`recipient`メソッドで取得することが出来ます。
`Addr<A>`の状態ではどのメッセージも送信することが出来ますが、`Recipient<M>`は一つのメッセージだけを送信することが出来るアドレスとなります。

どのような場合に便利かというと、複数のアクターに同じメッセージを送信する場合に`Addr<A>`形式だと`A`が実装アクターとなるため`Vec`や配列で持つことが出来ませんが、`Recipient<M>`は`M`がメッセージとなるため持つことが可能となる事があります。
また、Actixに含まれているプロセスシグナルアクター（`actix::actors::signal::ProcessSignals`）で使用しているような、特定の場合に自身へとメッセージを送信してもらう場合に自身の`Recipient`を送信する`Subscribe`メッセージ等といった使い方があります。


# Message
`Message`トレイトはアクターが送受信するメッセージを表します。
構造体を定義して`Message`トレイトを実装することでメッセージとなります。
`Message`トレイト実装時には関連型`Result`として戻り値を指定する必要があります。
また、`Message`トレイトはderiveして実装することも可能です。

```rust
// 通常の実装方法
struct TestMessage1;

impl Message for TestMessage1 {
    type Result = ();
}

// 戻り値がResult型の場合
struct TestMessage2;

impl Message for TestMessage2 {
    type Result = Result<u64, ::std::io::Error>;
}

// deriveした場合。戻り値が()の場合はrtype不要
#[derive(Message)]
struct TestMessage3;

// deriveで戻り値がResultの場合
#[derive(Message)]
#[rtype(result="Result<u64, ::std::io::Error>")]
struct TestMessage4;
```

## Handler
`Handler<M>`トレイトはアクターに実装し、メッセージの処理を行います。
`Handler<M>`トレイトも関連型として`Result`の指定が必要になります。
この`Result`は`Message`の`Result`とは少し異なり、`MessageResponse`トレイトを実装した型である必要があります。
一般的なプリミティブな型や`Result`、`Option`、`Box<Future<Item=I, Error=E>>`(`ResponseFuture<I, E>`)等はデフォルトで実装されていますが、自作の構造体等は通常`Result`等で包んで返却することになります。
また、一つのメッセージで同期処理する場合と非同期処理する場合がある時は`Response<I, E>`型を使用することも出来ます。
`ResponseFuture`や`Response`で返却する場合、メッセージの関連型は`Future`が解決された結果の`Result`である必要があります。
`Box<Future<Item=I, Error=E>>`では無い点に注意して下さい。

`Handler<M>`実装時に`handle`メソッドの記述が必要であり、メッセージを受信した際はこのメソッドが呼ばれます。


# Arbiter
`Arbiter`は一つのスレッドによる一つのイベントループを表します。
マルチスレッドで複数のイベントループを動作させたい場合は、`Arbiter`を作成してその上で`Actor`を動作させるようにします。

`Arbiter`は`System`を作成したタイミングで一つ作成され、`System::current().arbiter()`でSystem Arbiterのアドレスを取得することが出来ます。
特に何も指定せずに`Actor`を開始した場合は、開始処理が行われた際の`Arbiter`上で開始します。
よって、明示的に`Arbiter`を作成しない場合はシングルスレッドによる一つのイベントループで全ての`Actor`が動作することになります。

`Arbiter`を開始するには二つの方法があります。

一つ目は`Arbiter::new`で作成し、`Arbiter`のアドレスを取得する方法、二つ目は`Arbiter::start`で新規`Arbiter`上で開始したい`Actor`を指定する方法です。
`Arbiter::start`の戻り値は開始した`Actor`のアドレスになります。

現在の`Arbiter`とは別の作成済み`Arbiter`で`Actor`を開始したい場合は、対象の`Arbiter`のアドレスに`StartActor`メッセージを送信することで可能です。

どの`Actor`をどのスレッド(イベントループ)で動作させるかはプログラマがこの`Arbiter`を使用して制御することになります。
この起動部分以外ではスレッドについてあまり考えずにコードを記述できるのがActix(アクターモデル)の良い点だと思います。

## SyncArbiter
同じ`Actor`を複数別の`Arbiter`で動作させて、メッセージの処理を分散させたい場合があると思います。
その際はこの`SyncArbiter`を使用することが出来ます。

ただし、`SyncArbiter`で動かせるのは`SyncContext`を使用する`Actor`だけとなり、内部で非同期処理を行うことが出来ません。
具体的には`SyncContext`には`AsyncContext`トレイトが実装されていないため、`ActorFuture`等の非同期処理を実行することが出来ません。
そのため非同期処理を行った結果をselfに入れたりすることが出来ないという形です。

名前からも、主にCPUバウンドな処理を並列実行することを想定しているものと思われます。

`Actor`を初期化してから渡すことも出来ないため、初期化を非同期で行う必要があったり、失敗する可能性がある`Actor`の場合は使い辛かったりします。
個人的には非同期`Actor`を複数まとめて起動できるような仕組みがあれば嬉しいです。
※ざっと見たところAkkaにはそういうのもあるようですね。

# Supervisor
`Supervisor`は自身の上で動作する`Actor`を監視し、停止した際に自動的に再起動を行うことが出来ます。
注意点としては、ここでいう**停止**とは`Actor`の状態がStoppedになることを意味します。
panicが発生した際は現状拾われないようですので注意が必要です。

また、再起動と記載しましたが、実際に対象の`Actor`自体が新規に作成されるのではなく、同じ
`Actor`で`Supervised::restarting`及び`Actor::started`が呼び出される形になります。
よって、再起動時に必要な初期化処理をこの2か所で制御する必要があります。

`Actor`の停止は明示的に`Context::stop`をする必要があるため、復帰不可能な状態になった場合にpanicさせるのではなく`Context::stop`させるようにプログラムしておく必要があります。
panicが拾われないのは現状残念ですね。

尚、`Supervisor`上で動作させるためには`Supervised`トレイトを実装した`Actor`である必要があります。


# Registry
最後に`Registry`を紹介します。

ここまでの説明ではアクターの起動とその時に取得したアドレスを自身で管理し、各アクターに必要な他アクターのアドレスをcloneして渡したりする必要があります。
`Registry`は名前の通りアドレスを管理するレジストリとなり、グローバル的に使用可能なアドレス保管庫となります。

レジストリは二種類存在します。
一つは`SystemRegistry`であり、システム全体で一意なレジストリになります。
`SystemRegistry`は`System::current().registry()`で取得することが出来ます。
こちらは`SystemService`トレイトを実装したアクターを管理することが出来ます。

もう一つは`Registry`であり、`Arbiter`単位で一意なレジストリとなります。
`Registry`は`Arbiter::registry()`で取得することが出来ます。
こちらは`ArbiterService`トレイトを実装したアクターを管理することが出来ます。

`SystemService`トレイトと`ArbiterService`トレイトは殆ど同じものになります。
これらのトレイトの実装条件として、`Supervised`トレイトと`Default`トレイトが実装されている必要があります。

それぞれのレジストリには`get`と`set`というメソッドがあります。
`get`メソッドで対象アクターのアドレスを取得します。まだ対象アクターが起動していない場合は、`Default`でアクターが作成、開始されてそのアドレスが返ります。
事前に初期化などをしたい場合は、アクターを開始した後に`set`メソッドを使用してアドレスを登録することも可能です。

尚、アドレスを取得する際は上記に記載したレジストリの`get`メソッドを使用してもいいのですが、`SystemService`及び`ArbiterService`で実装されている`from_registry`が便利です。
`let addr = TestActor::from_registry()`というような形で`TestActor`のアドレスをレジストリから取得することが出来ますので、私はこの方法が好みです。


# 後書き
私にとって初めてのアクターモデルでしたが、最初の設定時以外はあまりスレッドを意識せずにプログラムが出来たというのが感想です。
元々Rustはデータ競合をコンパイル時に発見出来るので、通常のマルチスレッド言語におけるデータ競合が問題になることは少ないと思うので、その点で若干利点は少ないのかも知れません。

リモートサーバー分散が出来ないのが痛いところですが、そこが出来るようになるとRustの利用領域がより大きくなるかと思います。

Rustを学んで初期の頃は`Future`周りは非常にややこしいと思います。
Rust 2018でasync/await構文が来ると大きく改善されると期待しています。

Rustはとても良い言語だと思いますので広まって欲しいところです。
