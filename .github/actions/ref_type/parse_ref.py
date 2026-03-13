import re
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
    print(get_ref_type(sys.argv[1]))
