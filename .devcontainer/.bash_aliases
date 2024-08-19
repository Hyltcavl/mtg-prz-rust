gitfast() {
  if [ $# -eq 0 ]; then
    echo "Please give a commit message: gitfast <commit_message>"
    return 1
  fi
  commit_message="$1"
  git add -A
  git commit -m "$commit_message"
  git push
  echo "Changes committed and pushed successfully!"
}