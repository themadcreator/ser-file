import re
import os
import sys


def get_ref_type(ref):
    match = re.search(r"^refs/pull/([0-9]+)/merge$", ref)
    if match:
        return "pr"
    
    match = re.search(r"^refs/heads/(.+)$", ref)
    if match:
        return "branch"

    match = re.search(r"^refs/tags/(.+)$", ref)
    if match:
        sv_match = re.search(r"^v(?P<major>0|[1-9]\d*)\.(?P<minor>0|[1-9]\d*)\.(?P<patch>0|[1-9]\d*)(?:-(?P<prerelease>(?:0|[1-9]\d*|\d*[a-zA-Z-][0-9a-zA-Z-]*)(?:\.(?:0|[1-9]\d*|\d*[a-zA-Z-][0-9a-zA-Z-]*))*))?(?:\+(?P<buildmetadata>[0-9a-zA-Z-]+(?:\.[0-9a-zA-Z-]+)*))?$", match.group(1))
        if sv_match:
            if sv_match.group(4) is None:
                return "tag-semver-release"
            else:
                return "tag-semver-pre-release"
        else:
            return "tag"

    return "unknown"


if __name__ == "__main__":
    ref_type = get_ref_type(sys.argv[1])
    print("ref_type={}\n".format(ref_type))
    print("ref_type={}\n".format(ref_type), file=sys.stderr)


# # PR
# ref = "refs/pull/12/merge"

# # Branch
# ref = "refs/heads/[branch]"

# # Tag 
# ref = "refs/tags/v[semver]"

# semver = {
#     "major": match.group(1),
#     "minor": match.group(2),
#     "patch": match.group(3),
#     "pre-release": match.group(4),
#     "build-metadata": match.group(5),
# }:
# ref = "refs/pull/12/merge"
# ref = "refs/heads/[branch]"
# ref = "refs/tags/v[semver]"
