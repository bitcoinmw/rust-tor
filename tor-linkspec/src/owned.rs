//! Owned variants of [`ChanTarget`] and [`CircTarget`].

use std::net::SocketAddr;
use tor_llcrypto::pk;

use crate::{ChanTarget, CircTarget};

/// OwnedChanTarget is a summary of a [`ChanTarget`] that owns all of its
/// members.
#[derive(Debug, Clone)]
pub struct OwnedChanTarget {
	/// Copy of the addresses from the underlying ChanTarget.
	addrs: Vec<SocketAddr>,
	/// Copy of the ed25519 id from the underlying ChanTarget.
	ed_identity: Option<pk::ed25519::Ed25519Identity>,
	/// Copy of the rsa id from the underlying ChanTarget.
	rsa_identity: Option<pk::rsa::RsaIdentity>,
}

impl ChanTarget for OwnedChanTarget {
	fn addrs(&self) -> &[SocketAddr] {
		&self.addrs[..]
	}
	fn ed_identity(&self) -> Option<pk::ed25519::Ed25519Identity> {
		self.ed_identity
	}
	fn rsa_identity(&self) -> Option<pk::rsa::RsaIdentity> {
		self.rsa_identity
	}
	fn set_ed_identity(&mut self, ed: pk::ed25519::Ed25519Identity) {
		self.ed_identity = Some(ed);
	}
	fn set_rsa_identity(&mut self, rsa: pk::rsa::RsaIdentity) {
		self.rsa_identity = Some(rsa);
	}
}

impl OwnedChanTarget {
	/// Construct a new OwnedChanTarget from its parts.
	// TODO: Put this function behind a feature.
	pub fn new(
		addrs: Vec<SocketAddr>,
		ed_identity: Option<pk::ed25519::Ed25519Identity>,
		rsa_identity: Option<pk::rsa::RsaIdentity>,
	) -> Self {
		Self {
			addrs,
			ed_identity,
			rsa_identity,
		}
	}

	/// Construct a OwnedChanTarget from a given ChanTarget.
	pub fn from_chan_target<C>(target: &C) -> Self
	where
		C: ChanTarget + ?Sized,
	{
		OwnedChanTarget {
			addrs: target.addrs().to_vec(),
			ed_identity: target.ed_identity(),
			rsa_identity: target.rsa_identity(),
		}
	}
}

/// OwnedCircTarget is a summary of a [`CircTarget`] that owns all its
/// members.
#[derive(Debug, Clone)]
pub struct OwnedCircTarget {
	/// The fields from this object when considered as a ChanTarget.
	chan_target: OwnedChanTarget,
	/// The ntor key to use when extending to this CircTarget
	ntor_onion_key: pk::curve25519::PublicKey,
	/// The subprotocol versions that this CircTarget supports.
	protovers: tor_protover::Protocols,
}

impl OwnedCircTarget {
	/// Construct an OwnedCircTarget from a given CircTarget.
	pub fn from_circ_target<C>(target: &C) -> Self
	where
		C: CircTarget + ?Sized,
	{
		OwnedCircTarget {
			chan_target: OwnedChanTarget::from_chan_target(target),
			ntor_onion_key: *target.ntor_onion_key(),
			// TODO: I don't like having to clone here.  Our underlying
			// protovers parsing uses an Arc, IIRC.  Can we expose that here?
			protovers: target.protovers().clone(),
		}
	}
}

impl ChanTarget for OwnedCircTarget {
	fn addrs(&self) -> &[SocketAddr] {
		self.chan_target.addrs()
	}
	fn ed_identity(&self) -> Option<pk::ed25519::Ed25519Identity> {
		self.chan_target.ed_identity()
	}
	fn rsa_identity(&self) -> Option<pk::rsa::RsaIdentity> {
		self.chan_target.rsa_identity()
	}
	fn set_ed_identity(&mut self, ed: pk::ed25519::Ed25519Identity) {
		self.chan_target.set_ed_identity(ed);
	}
	fn set_rsa_identity(&mut self, rsa: pk::rsa::RsaIdentity) {
		self.chan_target.set_rsa_identity(rsa);
	}
}

impl CircTarget for OwnedCircTarget {
	fn ntor_onion_key(&self) -> &pk::curve25519::PublicKey {
		&self.ntor_onion_key
	}
	fn protovers(&self) -> &tor_protover::Protocols {
		&self.protovers
	}
}

#[cfg(test)]
mod test {
	use super::*;
	#[test]
	fn targetinfo() {
		let ti = OwnedChanTarget::new(
			vec!["127.0.0.1:11".parse().unwrap()],
			[42; 32].into(),
			[45; 20].into(),
		);

		let ti2 = OwnedChanTarget::from_chan_target(&ti);
		assert_eq!(ti.addrs(), ti2.addrs());
		assert_eq!(ti.ed_identity(), ti2.ed_identity());
		assert_eq!(ti.rsa_identity(), ti2.rsa_identity());
	}
}
