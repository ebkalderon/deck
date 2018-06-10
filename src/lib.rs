#[macro_use]
extern crate failure;
extern crate license_exprs;
extern crate ron;
extern crate semver;
#[macro_use]
extern crate serde;
extern crate url;
extern crate url_serde;

use std::iter::FromIterator;
use std::marker::PhantomData;
use std::vec::IntoIter;

use self::package_id::PackageId;
use self::package_id_spec::PackageIdSpec;

pub mod license;
pub mod manifest;
pub mod package_id;
pub mod package_id_spec;

#[derive(Debug)]
pub struct Installed<'p> {
    packages: IntoIter<PackageId<'p>>,
    marker: PhantomData<&'p ()>
}

impl<'p> Iterator for Installed<'p> {
    type Item = PackageId<'p>;

    fn next(&mut self) -> Option<Self::Item> {
        self.packages.next()
   }
}

#[derive(Debug, Fail)]
pub enum InstallError {
    #[fail(display = "package ID error")]
    PackageId,
    #[fail(display = "package ID error")]
    BuildFailed {
        package_id: String,
        msg: String,
    }
}

pub fn install<'p, P>(_pkgs: P) -> Result<Installed<'p>, ()>
where
    P: FromIterator<PackageIdSpec<'p>>
{
    Ok(Installed {
        packages: Vec::new().into_iter(),
        marker: PhantomData
    })
}
