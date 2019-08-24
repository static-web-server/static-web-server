def COLOR_MAP = ['SUCCESS': 'good', 'FAILURE': 'danger', 'UNSTABLE': 'danger', 'ABORTED': 'danger']

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
                sh 'rustc --version'
                sh 'cargo --version'
                sh 'cargo make --version'
                sh 'echo ""'
                sh 'echo "Install dependencies....."'
                sh 'echo ""'
                sh 'rustup target add x86_64-unknown-linux-musl'
	            sh 'cargo install --force cargo-make'
                sh 'rustup show'
            }
        }

        stage('Test') {
            steps {
                sh 'echo "There are no tests at the moment!"'
            }
        }

        stage('Build') {
            steps {
                echo "Running ${env.BUILD_ID} on ${env.JENKINS_URL}"
                sh 'cargo make --makefile Tasks.Prod.toml release'
                sh 'cargo make --makefile Tasks.Prod.toml docker_image'
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
                message: "*${currentBuild.currentResult}:* Job ${env.JOB_NAME} build ${env.BUILD_NUMBER}\n More info at: ${env.BUILD_URL}"
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
