extern crate chorus_lib;

use std::marker::PhantomData;

use chorus_lib::core::{
    ChoreoOp, Choreography, ChoreographyLocation, Faceted, FanInChoreography, FanOutChoreography,
    HCons, Located, LocationSet, LocationSetFoldable, Member, MultiplyLocated, Projector, Quire,
    Subset,
};
use chorus_lib::transport::local::{LocalTransport, LocalTransportChannelBuilder};

#[derive(ChoreographyLocation)]
struct Dealer;

#[derive(ChoreographyLocation)]
struct Player1;
#[derive(ChoreographyLocation)]
struct Player2;
#[derive(ChoreographyLocation)]
struct Player3;

fn read_i32() -> i32 {
    let mut input = String::new();
    std::io::stdin()
        .read_line(&mut input)
        .expect("Failed to read line");
    input.trim().parse::<i32>().expect("Failed to parse input")
}

struct Game<
    Players: LocationSet
        + Subset<HCons<Dealer, Players>, PlayersSubsetAll>
        + LocationSetFoldable<HCons<Dealer, Players>, Players, PlayersFoldable>,
    PlayersSubsetAll,
    PlayersFoldable,
> {
    phantom: PhantomData<(Players, PlayersSubsetAll, PlayersFoldable)>,
}

impl<
        Players: LocationSet
            + Subset<HCons<Dealer, Players>, PlayersSubsetAll>
            + LocationSetFoldable<HCons<Dealer, Players>, Players, PlayersFoldable>,
        PlayersSubsetAll,
        PlayersFoldable,
    > Game<Players, PlayersSubsetAll, PlayersFoldable>
{
    fn new<PlayerSubsetAll>(_: Players) -> Self
    where
        Players: Subset<HCons<Dealer, Players>, PlayerSubsetAll>,
    {
        Self {
            phantom: PhantomData {},
        }
    }
}

impl<
        Players: LocationSet
            + Subset<HCons<Dealer, Players>, PlayersSubsetAll>
            + LocationSetFoldable<HCons<Dealer, Players>, Players, PlayersFoldable>,
        PlayersSubsetAll,
        PlayersFoldable,
    > Choreography for Game<Players, PlayersSubsetAll, PlayersFoldable>
{
    type L = HCons<Dealer, Players>;

    fn run(self, op: &impl ChoreoOp<Self::L>) -> () {
        struct Collect<Players: LocationSet>(PhantomData<Players>);
        impl<Players: LocationSet> FanOutChoreography<i32> for Collect<Players> {
            type L = HCons<Dealer, Players>;
            type QS = Players;
            fn run<Q: ChoreographyLocation, QSSubsetL, QMemberL, QMemberQS>(
                &self,
                op: &impl ChoreoOp<Self::L>,
            ) -> Located<i32, Q>
            where
                Self::QS: Subset<Self::L, QSSubsetL>,
                Q: Member<Self::L, QMemberL>,
                Q: Member<Self::QS, QMemberQS>,
            {
                let card1 = op.locally(Dealer, |_| {
                    println!("Enter the first card for {:?}", Q::name());
                    read_i32()
                });
                op.comm(Dealer, Q::new(), &card1)
            }
        }
        let hand1 = op.fanout(Players::new(), Collect(PhantomData));

        struct Gather<
            'a,
            Players: LocationSet + Subset<HCons<Dealer, Players>, PlayersSubset>,
            PlayersSubset,
        > {
            hand1: &'a Faceted<i32, Players>,
            phantom: PhantomData<PlayersSubset>,
        }
        impl<
                'a,
                Players: LocationSet + Subset<HCons<Dealer, Players>, PlayersSubset>,
                PlayersSubset,
            > FanInChoreography<i32> for Gather<'a, Players, PlayersSubset>
        {
            type L = HCons<Dealer, Players>;
            type QS = Players;
            type RS = Players;
            fn run<Q: ChoreographyLocation, QSSubsetL, RSSubsetL, QMemberL, QMemberQS>(
                &self,
                op: &impl ChoreoOp<Self::L>,
            ) -> MultiplyLocated<i32, Self::RS>
            where
                Self::QS: Subset<Self::L, QSSubsetL>,
                Self::RS: Subset<Self::L, RSSubsetL>,
                Q: Member<Self::L, QMemberL>,
                Q: Member<Self::QS, QMemberQS>,
            {
                let x = op.locally(Q::new(), |un| *un.unwrap(self.hand1));
                let x = op.multicast::<Q, i32, Players, QMemberL, PlayersSubset>(
                    Q::new(),
                    <Self::RS>::new(),
                    &x,
                );
                x
            }
        }
        let on_the_table = op.fanin(
            Players::new(),
            Gather {
                hand1: &hand1,
                phantom: PhantomData,
            },
        );

        struct Choice<'a, Players: LocationSet> {
            hand1: &'a Faceted<i32, Players>,
            on_the_table: &'a MultiplyLocated<Quire<i32, Players>, Players>,
        }
        impl<'a, Players: LocationSet> FanOutChoreography<bool> for Choice<'a, Players> {
            type L = HCons<Dealer, Players>;
            type QS = Players;

            fn run<Q: ChoreographyLocation, QSSubsetL, QMemberL, QMemberQS>(
                &self,
                op: &impl ChoreoOp<Self::L>,
            ) -> Located<bool, Q>
            where
                Self::QS: Subset<Self::L, QSSubsetL>,
                Q: Member<Self::L, QMemberL>,
                Q: Member<Self::QS, QMemberQS>,
            {
                op.locally(Q::new(), |un| {
                    let hand1 = *un.unwrap(self.hand1);
                    let on_the_table = un.unwrap(self.on_the_table);
                    println!("My first card is: {}", hand1);
                    println!("On the table: {:?}", on_the_table);
                    println!("I'll ask for another? [True/False]");
                    let mut input = String::new();
                    std::io::stdin()
                        .read_line(&mut input)
                        .expect("Failed to read line");
                    let input = input.trim();
                    if input == "True" {
                        true
                    } else {
                        false
                    }
                })
            }
        }

        let wants_next_card = op.fanout(
            Players::new(),
            Choice {
                hand1: &hand1,
                on_the_table: &on_the_table,
            },
        );

        struct Collect2<'a, Players: LocationSet> {
            hand1: &'a Faceted<i32, Players>,
            wants_next_card: &'a Faceted<bool, Players>,
        }
        impl<'a, Players: LocationSet> FanOutChoreography<Vec<i32>> for Collect2<'a, Players> {
            type L = HCons<Dealer, Players>;
            type QS = Players;
            fn run<Q: ChoreographyLocation, QSSubsetL, QMemberL, QMemberQS>(
                &self,
                op: &impl ChoreoOp<Self::L>,
            ) -> Located<Vec<i32>, Q>
            where
                Self::QS: Subset<Self::L, QSSubsetL>,
                Q: Member<Self::L, QMemberL>,
                Q: Member<Self::QS, QMemberQS>,
            {
                struct Conclave<Player: ChoreographyLocation> {
                    hand1: Located<i32, Player>,
                    wants_next_card: Located<bool, Player>,
                }
                impl<Player: ChoreographyLocation> Choreography<Located<Vec<i32>, Player>> for Conclave<Player> {
                    type L = LocationSet!(Dealer, Player);

                    fn run(self, op: &impl ChoreoOp<Self::L>) -> Located<Vec<i32>, Player> {
                        let choice = op.broadcast(Player::new(), self.wants_next_card.clone());
                        if choice {
                            let card2 = op.locally(Dealer, |_| {
                                println!("Player {:?} wants another card", Player::name());
                                println!("Enter the second card for {:?}", Player::name());
                                read_i32()
                            });
                            let card2 = op.comm(Dealer, Player::new(), &card2);
                            op.locally(Player::new(), |un| {
                                vec![*un.unwrap(&self.hand1), *un.unwrap(&card2)]
                            })
                        } else {
                            op.locally(Player::new(), |un| vec![*un.unwrap(&self.hand1)])
                        }
                    }
                }
                let hand1 = op.locally(Q::new(), |un| *un.unwrap(self.hand1));
                let wants_next_card = op.locally(Q::new(), |un| *un.unwrap(self.wants_next_card));
                op.conclave(Conclave::<Q> {
                    hand1,
                    wants_next_card,
                })
                .flatten()
            }
        }
        let hand2 = op.fanout(
            Players::new(),
            Collect2 {
                hand1: &hand1,
                wants_next_card: &wants_next_card,
            },
        );
        let tbl_card = op.locally(Dealer, |_| {
            println!("Enter a single card for everyone ");
            read_i32()
        });
        let table_card = op.broadcast(Dealer, tbl_card);

        struct Outcome<'a, Players: LocationSet> {
            hand2: &'a Faceted<Vec<i32>, Players>,
            table_card: i32,
        }
        impl<'a, Players: LocationSet> FanOutChoreography<()> for Outcome<'a, Players> {
            type L = HCons<Dealer, Players>;
            type QS = Players;
            fn run<Q: ChoreographyLocation, QSSubsetL, QMemberL, QMemberQS>(
                &self,
                op: &impl ChoreoOp<Self::L>,
            ) -> Located<(), Q>
            where
                Self::QS: Subset<Self::L, QSSubsetL>,
                Q: Member<Self::L, QMemberL>,
                Q: Member<Self::QS, QMemberQS>,
            {
                op.locally(Q::new(), |un| {
                    let mut hand2 = un.unwrap(self.hand2).clone();
                    hand2.push(self.table_card);
                    println!("Final hands: {:?}", hand2);
                    let sum: i32 = hand2.iter().sum();
                    println!("My win result: {}", sum % 21 > 19);
                    return ();
                })
            }
        }

        op.fanout(
            Players::new(),
            Outcome {
                hand2: &hand2,
                table_card,
            },
        );
    }
}

fn main() {
    let transport_channel = LocalTransportChannelBuilder::new()
        .with(Dealer)
        .with(Player1)
        .with(Player2)
        .build();

    type Players = LocationSet!(Player1, Player2);

    let transport_dealer = LocalTransport::new(Dealer, transport_channel.clone());
    let transport_player1 = LocalTransport::new(Player1, transport_channel.clone());
    let transport_player2 = LocalTransport::new(Player2, transport_channel.clone());

    let dealer_projector = Projector::new(Dealer, transport_dealer);
    let player1_projector = Projector::new(Player1, transport_player1);
    let player2_projector = Projector::new(Player2, transport_player2);

    let mut handles = Vec::new();
    handles.push(std::thread::spawn(move || {
        dealer_projector.epp_and_run(Game::new(Players::new()));
    }));
    handles.push(std::thread::spawn(move || {
        player1_projector.epp_and_run(Game::new(Players::new()));
    }));
    handles.push(std::thread::spawn(move || {
        player2_projector.epp_and_run(Game::new(Players::new()));
    }));

    for handle in handles {
        handle.join().unwrap();
    }
}
