def COLOR_MAP = ['SUCCESS': 'good', 'FAILURE': 'danger', 'UNSTABLE': 'danger', 'ABORTED': 'danger']

def getBuildUser() {
    return currentBuild.rawBuild.getCause(Cause.UserIdCause).getUserId()
}

pipeline {
    agent {
        docker {
            image 'rust:1.37-slim'
        }
    }

    environment {
        CI = 'true'
        BUILD_USER = ''
    }

    stages {
        stage('Checkout') {
            steps {
                checkout scm
            }
        }

        stage('Init') {
            steps {
                script {
                    BUILD_USER = getBuildUser()
                }

                sh 'rustc --version'
                sh 'cargo --version'
                sh 'cargo make --version'
                sh 'make install'
                sh 'rustup show'
            }
        }

        stage('Test') {
            steps {
                sh 'make test'
            }
        }

        stage('Build') {
            steps {
                echo "Running ${env.BUILD_ID} on ${env.JENKINS_URL}"
                sh 'make release'
                sh 'make docker.image'
            }
        }

        stage('Deploy') {
            when {
              expression {
                currentBuild.result == null || currentBuild.result == 'SUCCESS'
              }
            }
            steps {
                // sh 'cargo release --skip-push --skip-publish beta'
                sh 'echo "Push current docker image..........."'
            }
        }
    }

    post {
        success {
            slackSend (
                channel: '#jenkins',
                color: "good",
                message: "*${currentBuild.currentResult}:* Job ${env.JOB_NAME} build ${env.BUILD_NUMBER} by ${env.BUILD_USER}\n More info at: ${env.BUILD_URL}"
            )
        }
        failure {
            slackSend (
                channel: '#jenkins',
                color: COLOR_MAP[currentBuild.currentResult],
                message: "*${currentBuild.currentResult}:* Job ${env.JOB_NAME} build ${env.BUILD_NUMBER} by ${env.BUILD_USER}\n More info at: ${env.BUILD_URL}"
            )
        }
    }
}
