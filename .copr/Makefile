# COPR "SCM" source type with build method `make srpm`.
# COPR calls: make -f <this file> srpm outdir=<dir> spec=<spec>
.PHONY: srpm
srpm:
	dnf -y install git rpm-build
	packaging/rpm/build-srpm.sh $(outdir)
