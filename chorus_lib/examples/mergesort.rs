use std::marker::PhantomData;
use std::thread::{self, JoinHandle};

use chorus_lib::core::{
    ChoreoOp, Choreography, ChoreographyLocation, Located, LocationSet, Projector,
};
use chorus_lib::transport::local::{LocalTransport, LocalTransportChannelBuilder};

#[derive(ChoreographyLocation)]
pub struct Alice;

#[derive(ChoreographyLocation)]
pub struct Bob;

#[derive(ChoreographyLocation)]
pub struct Carol;

pub struct Main;

impl Choreography for Main {
    type L = LocationSet!(Alice, Bob, Carol);

    fn run(self, op: &impl ChoreoOp<Self::L>) {
        let shuffled_list_at_alice = Located::local(vec![9, 1, 4, 7, 5, 2, 3, 0, 6, 8]);
        let sorted_list_at_alice = op.call(Sort::<Alice, Bob, Carol>::new(shuffled_list_at_alice));
        op.locally(Alice, |un| {
            let sorted_list = un.unwrap(&sorted_list_at_alice);
            println!("{sorted_list:?}");
        });
    }
}

pub struct Sort<A, B, C>
where
    A: ChoreographyLocation,
    B: ChoreographyLocation,
    C: ChoreographyLocation,
{
    list_at_a: Located<Vec<i64>, A>,
    phantom: PhantomData<(B, C)>,
}

impl<'a, A, B, C> Sort<A, B, C>
where
    A: ChoreographyLocation,
    B: ChoreographyLocation,
    C: ChoreographyLocation,
{
    pub const fn new(list: Located<Vec<i64>, A>) -> Self {
        Self {
            list_at_a: list,
            phantom: PhantomData,
        }
    }
}

impl<A, B, C> Choreography<Located<Vec<i64>, A>> for Sort<A, B, C>
where
    A: ChoreographyLocation,
    B: ChoreographyLocation,
    C: ChoreographyLocation,
{
    type L = LocationSet!(A, B, C);

    fn run(self, op: &impl ChoreoOp<Self::L>) -> Located<Vec<i64>, A> {
        #[allow(non_snake_case)]
        let (A, B, C) = (A::new(), B::new(), C::new());

        let is_list_small = op.broadcast(
            A,
            op.locally(A, |un| {
                let list = un.unwrap(&self.list_at_a);
                list.len() <= 1
            }),
        );

        if is_list_small {
            return self.list_at_a;
        }

        let pivot_at_a = op.locally(A, |un| {
            let list = un.unwrap(&self.list_at_a);
            list.len() / 2
        });

        let prefix_at_a = op.locally(A, |un| {
            let list = un.unwrap(&self.list_at_a);
            let pivot = un.unwrap(&pivot_at_a);
            list[..*pivot].to_vec()
        });
        let prefix_at_b = op.comm(A, B, &prefix_at_a);
        let sorted_prefix_at_b = op.call(Sort::<B, C, A>::new(prefix_at_b));

        let suffix_at_a = op.locally(A, |un| {
            let list = un.unwrap(&self.list_at_a);
            let pivot = un.unwrap(&pivot_at_a);
            list[*pivot..].to_vec()
        });
        let suffix_at_c = op.comm(A, C, &suffix_at_a);
        let sorted_suffix_at_c = op.call(Sort::<C, A, B>::new(suffix_at_c));

        op.call(Merge::new(sorted_prefix_at_b, sorted_suffix_at_c))
    }
}

pub struct Merge<B, C>
where
    B: ChoreographyLocation,
    C: ChoreographyLocation,
{
    prefix_at_b: Located<Vec<i64>, B>,
    suffix_at_c: Located<Vec<i64>, C>,
}

impl<B, C> Merge<B, C>
where
    B: ChoreographyLocation,
    C: ChoreographyLocation,
{
    pub const fn new(prefix_at_b: Located<Vec<i64>, B>, suffix_at_c: Located<Vec<i64>, C>) -> Self {
        Self {
            prefix_at_b,
            suffix_at_c,
        }
    }
}

impl<A, B, C> Choreography<Located<Vec<i64>, A>> for Merge<B, C>
where
    A: ChoreographyLocation,
    B: ChoreographyLocation,
    C: ChoreographyLocation,
{
    type L = LocationSet!(A, B, C);

    fn run(self, op: &impl ChoreoOp<Self::L>) -> Located<Vec<i64>, A> {
        #[allow(non_snake_case)]
        let (A, B, C) = (A::new(), B::new(), C::new());

        let is_prefix_empty = op.broadcast(
            B,
            op.locally(B, |un| {
                let prefix = un.unwrap(&self.prefix_at_b);
                prefix.is_empty()
            }),
        );

        if is_prefix_empty {
            let suffix_at_c = op.locally(C, |un| un.unwrap(&self.suffix_at_c).to_vec());
            return op.comm(C, A, &suffix_at_c);
        }

        let is_suffix_empty = op.broadcast(
            C,
            op.locally(C, |un| {
                let suffix = un.unwrap(&self.suffix_at_c);
                suffix.is_empty()
            }),
        );

        if is_suffix_empty {
            let prefix_at_b = op.locally(B, |un| un.unwrap(&self.prefix_at_b).to_vec());
            return op.comm(B, A, &prefix_at_b);
        }

        let head_of_prefix_at_b = op.locally(B, |un| un.unwrap(&self.prefix_at_b)[0]);
        let head_of_prefix_at_c = op.comm(B, C, &head_of_prefix_at_b);
        let head_of_suffix_at_c = op.locally(C, |un| un.unwrap(&self.suffix_at_c)[0]);

        let is_head_of_prefix_smaller = op.broadcast(
            C,
            op.locally(C, |un| {
                let head_of_prefix = un.unwrap(&head_of_prefix_at_c);
                let head_of_suffix = un.unwrap(&head_of_suffix_at_c);
                head_of_prefix <= head_of_suffix
            }),
        );

        if is_head_of_prefix_smaller {
            let head_of_prefix_at_a = op.comm(B, A, &head_of_prefix_at_b);
            let prefix_at_b = op.locally(B, |un| {
                let prefix = un.unwrap(&self.prefix_at_b);
                prefix[1..].to_vec()
            });
            let partial_at_a = op.call(Merge::new(prefix_at_b, self.suffix_at_c));
            op.locally(A, |un| {
                let partial = un.unwrap(&partial_at_a);
                let head_of_prefix = un.unwrap(&head_of_prefix_at_a);
                [head_of_prefix]
                    .into_iter()
                    .chain(partial)
                    .copied()
                    .collect()
            })
        } else {
            let head_of_suffix_at_a = op.comm(C, A, &head_of_suffix_at_c);
            let suffix_at_c = op.locally(C, |un| {
                let suffix = un.unwrap(&self.suffix_at_c);
                suffix[1..].to_vec()
            });
            let partial_at_a = op.call(Merge::new(self.prefix_at_b, suffix_at_c));
            op.locally(A, |un| {
                let partial = un.unwrap(&partial_at_a);
                let head_of_suffix = un.unwrap(&head_of_suffix_at_a);
                [head_of_suffix]
                    .into_iter()
                    .chain(partial)
                    .copied()
                    .collect()
            })
        }
    }
}

fn main() {
    let mut handles = Vec::new();
    let transport_channel = LocalTransportChannelBuilder::new()
        .with(Alice)
        .with(Bob)
        .with(Carol)
        .build();

    {
        let transport = LocalTransport::new(Alice, transport_channel.clone());
        handles.push(thread::spawn(move || {
            let projector = Projector::new(Alice, transport);
            projector.epp_and_run(Main);
        }));
    }

    {
        let transport = LocalTransport::new(Bob, transport_channel.clone());
        handles.push(thread::spawn(move || {
            let projector = Projector::new(Bob, transport);
            projector.epp_and_run(Main);
        }));
    }

    {
        let transport = LocalTransport::new(Carol, transport_channel.clone());
        handles.push(thread::spawn(move || {
            let projector = Projector::new(Carol, transport);
            projector.epp_and_run(Main);
        }));
    }

    handles.into_iter().try_for_each(JoinHandle::join).unwrap();
}
