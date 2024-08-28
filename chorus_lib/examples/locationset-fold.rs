use chorus_lib::core::{
    ChoreographyLocation, LocationSet, LocationSetFoldable, LocationSetFolder, Member, Subset,
};

#[derive(ChoreographyLocation)]
struct Alice;
#[derive(ChoreographyLocation)]
struct Bob;
#[derive(ChoreographyLocation)]
struct Carol;

fn main() {
    type L = LocationSet!(Alice, Bob, Carol);
    type QS = LocationSet!(Bob, Carol);
    struct F;
    impl LocationSetFolder<String> for F {
        type L = L;
        type QS = QS;
        fn f<Q: ChoreographyLocation, QSSubsetL, QMemberL, QMemberQS>(
            &self,
            acc: String,
            curr: Q,
        ) -> String
        where
            Self::QS: Subset<Self::L, QSSubsetL>,
            Q: Member<Self::L, QMemberL>,
            Q: Member<Self::QS, QMemberQS>,
        {
            let mut x = acc.clone();
            x.push_str(Q::name());
            x
        }
    }
    let x = QS::foldr(F {}, String::new());
    println!("{}", x);
}
